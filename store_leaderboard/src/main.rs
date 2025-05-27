use std::io::{BufRead, BufReader, Write};
use std::fs::{File, OpenOptions};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use redis::{AsyncCommands, RedisResult};

type Leaderboard = Vec<(String, usize)>;
type Error = Box<dyn std::error::Error>;

mod leaderboard;
mod config;

// see leaderboard.rs
async fn get_leaderboard() -> Result<Leaderboard, Error> {    
    let data = leaderboard::get_data().await?; 
    Ok(data)
}

// takes a Leaderboard: Vec<(String, usize)> and uploads it to the DB.
// some additional checks added to stabalize displayed values from wowaudit api.
async fn store_leaderboard(leaderboard: Leaderboard) -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;

    // Read existing leaderboard into a HashMap
    let existing_lb = read_leaderboard().await?;

    // Process incoming leaderboard into a HashMap, keeping the highest value per name
    let mut incoming_lb = HashMap::new();
    for (name, keys) in leaderboard {
        incoming_lb.entry(name)
            .and_modify(|e| if keys > *e { *e = keys })
            .or_insert(keys);
    }

    // Update Redis only if new value is higher
    for (name, new_keys) in incoming_lb {
        match existing_lb.get(&name) {
            Some(existing) => { 
                if new_keys > *existing  {
                    let _: () = conn.hset("leaderboard", &name, new_keys).await?;
                    println!("[UPDATE] name: {} | existing = {} -> new = {}", &name, existing, new_keys);

                } else {
                    println!("[CACHE-EXCEPTION] name: {} | existing = {} > new = {}", &name, existing, new_keys);
                }
            }
            None => {
                let _: () = conn.hset("leaderboard", &name, new_keys).await?;
                println!("[CREATE] name: {} | new = {}", &name, new_keys);
            }
        }
    }

    Ok(())
}

// Reads all keys in "leaderboard" into a hashmap
async fn read_leaderboard() -> redis::RedisResult<HashMap<String, usize>> {
    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;
    
    let hash_map: HashMap<String, usize> = conn.hgetall("leaderboard").await?;
    Ok(hash_map)
}

async fn remove_from_leaderboard(names: &[String]) -> RedisResult<()> {
    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;
    
    for name in names {
        let removed: i32 = conn.hdel("leaderboard", name).await?;
        if removed > 0 {
            println!("[REMOVE] name: {} was removed.", name);
        } else {
            println!("[REMOVE ERROR] name: {} not found.", name);
        }
    }
    Ok(())

}


// append leaderboard state to leaderboard.log
async fn backup_leaderboard() -> RedisResult<()> {
    let leaderboard = read_leaderboard().await?;
    let time_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timestamp = format!("[{}] Leaderboard State Backup:\n", time_start.as_secs());

    let mut log_entry = String::new();
    log_entry.push_str(&timestamp);
    for (name, keys) in leaderboard {
        log_entry.push_str(&format!("{}: {}\n", name, keys));
    }
    log_entry.push('\n');
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("leaderboard.log")?;
    file.write_all(log_entry.as_bytes())?;
    Ok(())
}

// delete leaderboard set and append state to leaderboard.log
async fn delete_leaderboard() -> RedisResult<()> {
    backup_leaderboard().await?;

    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;
   
    let deleted: i32 = conn.del("leaderboard").await?;
    if deleted > 0 {
        println!("[DELETE-SET] leaderboard has been removed.");
    } else {
        println!("[DELETE-SET ERROR] leaderboard set not found.");
    }

    Ok(())

}

// recover leaderboard state from leaderboard.log given a timestamp
async fn restore_leaderboard(timestamp: &str) -> RedisResult<()> {
    let file = File::open("leaderboard.log")?;
    let reader = BufReader::new(file);

    let target_header = format!("[{}] Leaderboard State Backup:", timestamp);
    let mut found = false;
    let mut restore: HashMap<String, usize> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            found = false;
            continue;
        }
        if !found {
            if line.trim() == target_header {
                found = true;
            }
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_string();
            if let Ok(keys) = value.trim().parse::<usize>() {
                restore.insert(name, keys);
            }
        }
    }
    if restore.is_empty() {
        println!("[RESTORE ERROR] No data found for timestamp '{}'.", timestamp);
        return Ok(());
    }
    println!("[RESTORE] {} entries found, clearing current leaderboard...", restore.len());
    delete_leaderboard().await?;

    // Re-insert recovered data
    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;
    for (name, keys) in restore {
        let _: () = conn.hset("leaderboard", name, keys).await?;
    }

    println!("[RESTORE] Successfully restored leaderboard state from [{}].", timestamp);
    Ok(())

}


#[tokio::main]
async fn main() -> Result<(), Error>{
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {

        Some("--remove") => {
            let raw_names: Vec<String> = args.collect();
            // support  --remove foo,bar and --remove foo bar 
            let names: Vec<String> = raw_names
                .into_iter()
                .flat_map(|s| s.split(',').map(|x| x.trim().to_string()).collect::<Vec<_>>())
                .filter(|s| !s.is_empty())
                .collect();
            if names.is_empty() {
                eprintln!("[ERROR] Expected names. Example: cargo run -- --remove foo bar OR --remove foo,bar")
            } else {
                remove_from_leaderboard(&names).await?;
            }
        } 

        Some("--delete") => {
            match args.next().as_deref() {
                Some("leaderboard") => {
                    delete_leaderboard().await?;
                }
                Some(set) => {
                    eprintln!("{} set not-found in redis", set);
                }
                None => {
                    eprintln!("Missing delete target (example: `--remove leaderboard`)");
                }
            }
        }

        Some("--restore") => {
            if let Some(timestamp) = args.next() {
                restore_leaderboard(&timestamp).await?;
            } else {
                eprintln!("Missing timestamp to restore.");
            }
        }

        Some(flag) => {
            eprintln!("Unknown flag: {}", flag);
        }

        None => {
            let lb = get_leaderboard().await?;
            store_leaderboard(lb).await?;
        }
    }
    Ok(())
}


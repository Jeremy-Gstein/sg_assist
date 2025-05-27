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

#[tokio::main]
async fn main() -> Result<(), Error>{
    let mut args = std::env::args().skip(1);
    if let Some(flag) = args.next() {
        if flag == "--remove" {
            let raw_names: Vec<String> = args.collect();
            // support  --remove foo,bar and --remove foo bar 
            let names: Vec<String> = raw_names
                .into_iter()
                .flat_map(|s| s.split(',').map(|x| x.trim().to_string()).collect::<Vec<_>>())
                .filter(|s| !s.is_empty())
                .collect();
            if names.is_empty() {
                eprint!("[ERROR] Expected names. Example: cargo run -- --remove foo bar OR --remove foo,bar")
            } else {
                remove_from_leaderboard(&names).await?;
            }
            return Ok(());
        } else {
            eprintln!("Unknown flag: {}", flag);
            return Ok(());
        }
    }
    let lb = get_leaderboard().await?;
    store_leaderboard(lb).await?;
    Ok(())
}

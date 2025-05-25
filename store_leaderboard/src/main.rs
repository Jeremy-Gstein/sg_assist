use std::collections::HashMap;
use redis::AsyncCommands;

type Leaderboard = Vec<(String, usize)>;
type Error = Box<dyn std::error::Error>;

mod leaderboard;
mod config;

async fn get_leaderboard() -> Result<Leaderboard, Error> {    
    let data = leaderboard::get_data().await?; 
    Ok(data)
}

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

async fn read_leaderboard() -> redis::RedisResult<HashMap<String, usize>> {
    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;
    
    let hash_map: HashMap<String, usize> = conn.hgetall("leaderboard").await?;
    Ok(hash_map)
}

#[tokio::main]
async fn main() -> Result<(), Error>{
    let lb = get_leaderboard().await?;
    store_leaderboard(lb).await?;
    Ok(())
}

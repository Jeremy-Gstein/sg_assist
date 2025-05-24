use redis::AsyncCommands;

mod config;
mod leaderboard;

type Leaderboard = Vec<(String, usize)>;
type Error = Box<dyn std::error::Error>;


async fn get_leaderboard() -> Result<Leaderboard, Error> {
    
    let data = leaderboard::get_data().await?; 
    Ok(data)
}

async fn store_leaderboard(leaderboard: Leaderboard)-> redis::RedisResult<()> {

    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;

    for (name, keys) in leaderboard {
        let _: () = conn.hset("leaderboard", name, keys).await?;
    }
    Ok(())
}

async fn read_leaderboard() -> redis::RedisResult<Leaderboard> {
    let client = redis::Client::open("redis://:sgdbadmin@127.0.0.1/")?;
    let mut conn = client.get_multiplexed_tokio_connection().await?;
    
    let vec_leaderboard: Vec<(String, usize)> = conn.hgetall("leaderboard").await?;
    Ok(vec_leaderboard)
}


#[tokio::main]
async fn main() -> Result<(), Error>{
    let lb = get_leaderboard().await?;
    store_leaderboard(lb).await?;
    //let rlb = read_leaderboard().await?;
    //dbg!(rlb);

    Ok(())

}

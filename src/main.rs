use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod config;
mod leaderboard;
mod verbose;
mod period;
mod charactermeta;

#[derive(Debug)]
pub struct DungeonRun {
    level: i32,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DungeonDone {
    level: i32,
    dungeon: i32,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterData {
    dungeons_done: Option<Vec<DungeonDone>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Character {
    id: i32,
    name: String,
    realm: String,
    data: Option<CharacterData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    period: i32,
    characters: Vec<Character>,
}



fn get_dungeon_mapping() -> HashMap<i32, String> {
    let mut mapping = HashMap::new();
    // season 1 
    mapping.insert(375, "Mists of Tirna Scithe".to_string());
    mapping.insert(502, "City of Threads".to_string());
    mapping.insert(503, "Ara-Kara, City of Echoes".to_string());
    mapping.insert(505, "Dawnbreaker".to_string());
    mapping.insert(507, "Grim Batol".to_string());
    mapping.insert(353, "Seige of Boralus".to_string());
    mapping.insert(501, "Stonevault".to_string());
    mapping.insert(376, "Nercrotic Wake".to_string());
    // season 2
    mapping.insert(506, "Cinderbrew Meadery".to_string());
    mapping.insert(504, "Darkflame Cleft".to_string());
    mapping.insert(370, "Mechagon Workshop".to_string());
    mapping.insert(525, "Floodgate".to_string());
    mapping.insert(499, "Priory of Sacred Flame".to_string());
    mapping.insert(247, "Motherload".to_string());
    mapping.insert(500, "Rookery".to_string());
    mapping.insert(382, "Theater of Pain".to_string());
    mapping
}


fn rename_dungeon(id: i32, mapping: &HashMap<i32,String>) -> Option<String> {
    mapping.get(&id).cloned()
}

async fn fetch_endpoint(token: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let url = format!("https://www.wowaudit.com/v1/historical_data");
    let client = reqwest::Client::new();
 
    let res = client
        .get(&url)
        .header("Accept", "application/json")
        .header("Authorization", token)
        .send()
        .await?
        .error_for_status()?;
    let body = res.text().await?;
    serde_json::from_str(&body).map_err(Into::into)
}

// +---------------------------------+
// |   DEBUG FOR THE DEBUG SCRIPT    |
// +---------------------------------+

async fn debug() {
    // if let Err(e) = leaderboard::get_data().await {
    //     eprint!("Error: {}", e);
    // }
    //
    if let Err(e) = charactermeta::sorted_date().await {
        eprint!("Error: {}", e);
    }

}


async fn leaderboard() {
    if let Err(e) = leaderboard::get_data().await {
        eprint!("Error: {}", e);
    }
}

async fn verbose_character_data() {
    if let Err(e) = verbose::call_wowaudit_api(true).await {
        eprintln!("Error: {}", e);
    }
}

async fn character_data() {
    if let Err(e) = verbose::call_wowaudit_api(false).await {
        eprintln!("Error: {}", e);
    }
}

async fn season_leaderboard() {
    if let Err(e) = period::get_period_data().await {
        eprintln!("Error: {}", e);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env
    println!("SG Assistant Debug for /keysdone");
    let args: Vec<String> = std::env::args().skip(1).collect();
    if ! args.is_empty() && args.contains(&"--l".to_string()) {
        leaderboard().await;
    } else if ! args.is_empty() && args.contains(&"--d".to_string()) {
        debug().await;
    } else if ! args.is_empty() && args.contains(&"--c".to_string()) {
        character_data().await;
    } else if ! args.is_empty() && args.contains(&"--a".to_string()) {
        season_leaderboard().await;
    } else if ! args.is_empty() && args.contains(&"--v".to_string()) {
        verbose_character_data().await;
    } else {
        loop {
            println!("\nSelect an option:");
            println!("[L] Weekly Leaderboard");
            println!("[A] Season Leaderboard");
            println!("[C] Character Data (Weekly)");
            println!("[V] Verbose Character Data (Weekly)");
            println!("[Q] Quit");
            let mut selection = String::new();
            std::io::stdin().read_line(&mut selection)?;
            let selection = selection.trim().to_uppercase();
            match selection.as_str() {
                "V" => {
                    println!("\nRunning with verbose character data..\n");
                    verbose_character_data().await;
                }
                "C" => {
                    println!("\nRunning with character data..\n");
                    character_data().await;
                }
                "L" => {
                    println!("\nRunning with leaderboard data..\n");
                    leaderboard().await;
                }
                "A" => {
                    println!("\nRunning with season leaderboard data..\n");
                    season_leaderboard().await;
                }
                "Q" => {
                    println!("\nExiting.. o/\n");
                    break; 
                }
                _ => {
                    println!("Invalid input. Please enter: [V] [C] [A] or [Q] to exit");
                }
            }
        }
    }
    Ok(())
}

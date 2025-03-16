use reqwest;
use serde_json::Value;
use crate::Context;
use crate::Error;
use std::collections::HashMap;

async fn fetch_character_data() -> Result<Value, reqwest::Error> {
    let token = std::env::var("WOWAUDIT_TOKEN").expect("missing WOWAUDIT_TOKEN");
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.wowaudit.com/v1/historical_data")
        .header("Authorization", token)
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(response)
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

fn rename_dungeon(id: i32, mapping: &HashMap<i32, String>) -> String {
    mapping.get(&id).cloned().unwrap_or_else(|| format!("Unknown Dungeon (ID: {})", id))
}

/// Search for a character's weekly completed keys (must be on the wowaudit use /roster)
#[poise::command(slash_command, broadcast_typing)]
pub async fn mykeys(
    ctx: Context<'_>,
    #[description = "Character name to search for"] name: String,
) -> Result<(), Error> {
    let response = fetch_character_data().await?;
    let mut msg_send = String::new();
    let dungeon_mapping = get_dungeon_mapping();

    if let Some(characters) = response["characters"].as_array() {
        if let Some(character) = characters.iter().find(|c| c["name"].as_str().unwrap_or("").to_lowercase() == name.to_lowercase()) {
            let char_name = character["name"].as_str().unwrap_or("Unknown");
            let empty_vec = Vec::new();
            let dungeons_done = character["data"]["dungeons_done"].as_array().unwrap_or(&empty_vec);
            msg_send.push_str(&format!("## Mythic+ Data for {}\n", char_name));
            let valid_dungeons: Vec<_> = dungeons_done
                .iter()
                .filter(|d| d["level"].as_u64().unwrap_or(0) >= 1)
                .collect();
            
            let dungeon_count = valid_dungeons.len();
            msg_send.push_str(&format!("### Total Keys Completed: {}\n", dungeon_count));
            let mut index = 0; 
            for dungeon in valid_dungeons {
                let dungeon_id = dungeon["dungeon"].as_u64().unwrap_or(0);
                let level = dungeon["level"].as_u64().unwrap_or(0);
                let dungeon_name = rename_dungeon(dungeon_id.try_into().unwrap(), &dungeon_mapping);
                // print 4 dungeons per new line
                if index >= 3 {
                    msg_send.push_str(&format!("+{} {} |\n", level, dungeon_name));
                    index = 0;
                } else {
                    msg_send.push_str(&format!("+{} {} | ", level, dungeon_name));
                    index += 1;
                }
            }
        } else {
            msg_send = format!("No data found for player: {}", name);
        }
    } else {
        msg_send = "No data available.".to_string();
    }

    ctx.say(msg_send).await?;

    Ok(())
}

use reqwest;
use serde_json::Value;
use crate::Context;
use crate::Error;

pub async fn fetch_character_data() -> Result<Value, reqwest::Error> {
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

/// Get this weeks keystone completion leaderboard
#[poise::command(slash_command, broadcast_typing)]
pub async fn keysdone(ctx: Context<'_>) -> Result<(), Error> {
    let response = fetch_character_data().await?;
    let mut msg_send = String::new();

    if let Some(characters) = response["characters"].as_array() {
        let mut sorted_characters: Vec<_> = characters
            .iter()
            .filter_map(|character| {
                let char_name = character["name"].as_str()?;
                let dungeons_done = character["data"]["dungeons_done"].as_array()?;
                let valid_dungeons: Vec<_> = dungeons_done
                    .iter()
                    .filter(|d| d["level"].as_u64().unwrap_or(0) >= 1)
                    .collect();
                let dungeon_count = valid_dungeons.len();
                if dungeon_count > 0 {
                    Some((char_name, valid_dungeons, dungeon_count))
                } else {
                    None
                }
            })
            .collect();

        sorted_characters.sort_by(|a, b| b.2.cmp(&a.2));
        msg_send.push_str("## | Mythic Plus Leaderboard |\n");
        for (name, _dungeons, dungeon_count) in sorted_characters {
            msg_send.push_str(&format!("### > - {} ~ Keys completed: {}\n", name, dungeon_count));
            //for dungeon in dungeons {
            //    let dungeon_id = dungeon["dungeon"].as_u64().unwrap_or(0);
            //    let level = dungeon["level"].as_u64().unwrap_or(0);
            //    msg_send.push_str(&format!("|*|+{} Dungeon ID#: {}\n", level, dungeon_id));
            //}
            //msg_send.push_str("\n"); // Add a blank line between characters
        }
        msg_send.push_str(&format!("Started Tracking Last Weekly Reset: <t:1738076400:R>"))
    } else {
        msg_send = "No character data available.".to_string();
    }

    ctx.say(msg_send).await?;

    Ok(())
}

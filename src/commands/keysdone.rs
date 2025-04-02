use reqwest;
use serde_json::Value;
use crate::Context;
use crate::Error;
use chrono::{DateTime, Utc};

// Fetch the current period ID from Raider.IO API
async fn fetch_period_id() -> Result<Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://raider.io/api/v1/periods")
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(response)
}

// Fetch WoWAudit Roster Mythic+ Data
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

/// Get this week's keystone completion leaderboard
#[poise::command(slash_command, broadcast_typing)]
pub async fn keysdone(ctx: Context<'_>) -> Result<(), Error> {
    let get_id = fetch_period_id().await?;
    let response = fetch_character_data().await?;

    // Parse the starting date of the current period
    let date_str = get_id["periods"][0]["current"]["start"].as_str();
    let datetime = DateTime::parse_from_rfc3339(date_str.expect("JSON Value not found"))
        .expect("Failed to parse timestamp")
        .with_timezone(&Utc);
    let unix_timestamp = datetime.timestamp();
    let formatted_date = format!("<t:{}:R>", unix_timestamp);

    let embed_description: String;
    let total_runs: usize;
    let mut leaderboard: Vec<(String, usize)> = Vec::new();

    // Extract and sort the character data
    if let Some(characters) = response["characters"].as_array() {
        total_runs = characters
            .iter()
            .filter_map(|character| character["data"]["dungeons_done"].as_array())
            .flatten()
            .filter(|d| d["level"].as_u64().unwrap_or(0) >= 1)
            .count();

        for character in characters {
            if let (Some(name), Some(dungeons_done)) = (
                character["name"].as_str(),
                character["data"]["dungeons_done"].as_array(),
            ) {
                let valid_dungeons: Vec<_> = dungeons_done
                    .iter()
                    .filter(|d| d["level"].as_u64().unwrap_or(0) >= 1)
                    .collect();
                let dungeon_count = valid_dungeons.len();

                if dungeon_count > 0 {
                    leaderboard.push((name.to_string(), dungeon_count));
                }
            }
        }

        // Sort leaderboard by dungeon count in descending order
        leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
        embed_description = leaderboard
            .iter()
            .take(8) // Display only the top 8 players
            .map(|(name, count)| format!("- **{}**: {} keys completed", name, count))
            .collect::<Vec<_>>()
            .join("\n");
    } else {
        total_runs = 0;
        embed_description = "No data available.".to_string();
    }

    // Create the embed for the leaderboard
    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title("Mythic+ Leaderboard")
        .description(embed_description)
        .field("Total Runs", total_runs.to_string(), false)
        .field("Started Tracking Since", formatted_date, false)
        .colour(poise::serenity_prelude::Colour::from_rgb(114, 137, 218)); // Embed color

    // Send the embed as a reply
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}


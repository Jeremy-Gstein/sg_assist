use reqwest;
use serde_json::Value;
use crate::Context;
use crate::Error;
use chrono::{DateTime, Utc};
use crate::paginate;

/// Fetch the current period ID from Raider.IO API
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

/// Fetch WoWAudit Roster Mythic+ Data
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

/// Get this week's keystone completion leaderboard, with pagination
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

    let mut leaderboard: Vec<(String, usize)> = Vec::new();
    let mut total_runs: usize = 0;

    // Extract and sort the character data
    if let Some(characters) = response["characters"].as_array() {
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
                    total_runs += dungeon_count;
                }
            }
        }

        // Sort leaderboard by dungeon count in descending order
        leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
    }

    // Generate paginated pages (chunks of 8 entries)
    let string_pages: Vec<String> = leaderboard
        .chunks(8)
        .enumerate()
        .map(|(page_idx, chunk)| {
            let entries = chunk
                .iter()
                .map(|(name, count)| format!("- **{}**: {} keys completed", name, count))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "**Mythic+ Leaderboard - Page {}/{}**\n\n{}\n\n**Total Runs:** {}\n**Started Tracking Since:** {}",
                page_idx + 1,
                (leaderboard.len() + 7) / 8, // Calculate total pages
                entries,
                total_runs,
                formatted_date
            )
        })
        .collect();

    // Convert Vec<String> to Vec<&str> for paginate
    let pages: Vec<&str> = string_pages.iter().map(String::as_str).collect();

    // Use paginate to send paginated embeds
    paginate(ctx, &pages).await?;



    Ok(())
}


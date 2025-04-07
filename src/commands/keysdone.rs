use reqwest;
use serde_json::Value;
use crate::Context;
use crate::Error;
use chrono::{DateTime, Utc};
// use poise::serenity_prelude as serenity;
use futures::future::BoxFuture;
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

/// Function to generate pages from data
fn generate_pages(data: &Value, period_info: &Value) -> Result<Vec<String>, Error> {
    let date_str = period_info["periods"][0]["current"]["start"]
        .as_str()
        .ok_or("Missing period start date")?;
    let datetime = DateTime::parse_from_rfc3339(date_str)?
        .with_timezone(&Utc);
    let unix_timestamp = datetime.timestamp();
    let formatted_date = format!("<t:{}:R>", unix_timestamp);

    let mut leaderboard: Vec<(String, usize)> = Vec::new();
    let mut total_runs: usize = 0;

    if let Some(characters) = data["characters"].as_array() {
        for character in characters {
            if let (Some(name), Some(dungeons_done)) = (
                character["name"].as_str(),
                character["data"]["dungeons_done"].as_array(),
            ) {
                let dungeon_count = dungeons_done
                    .iter()
                    .filter(|d| d["level"].as_u64().unwrap_or(0) >= 1)
                    .count();

                if dungeon_count > 0 {
                    leaderboard.push((name.to_string(), dungeon_count));
                    total_runs += dungeon_count;
                }
            }
        }
        leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
    }

    Ok(leaderboard
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
                (leaderboard.len() + 7) / 8,
                entries,
                total_runs,
                formatted_date
            )
        })
        .collect())
}



/// Get this week's keystone completion leaderboard
#[poise::command(slash_command, broadcast_typing)]
pub async fn keysdone(ctx: Context<'_>) -> Result<(), Error> {
    // Initial fetch
    let period_info = fetch_period_id().await?;
    let initial_data = fetch_character_data().await?;
    let initial_pages = generate_pages(&initial_data, &period_info)?;

    // Create refresh closure with static lifetime
    let refresh_closure = {
        let http = ctx.serenity_context().http.clone();
        move || {
            let _http = http.clone();
            Box::pin(async move {
                let new_data = fetch_character_data().await?;
                let new_period = fetch_period_id().await?;
                generate_pages(&new_data, &new_period)
            }) as BoxFuture<'static, Result<Vec<String>, Error>>
        }
    };

    paginate(ctx, initial_pages, refresh_closure).await?;
    Ok(())
}


use crate::{Context, Error};
use poise::{serenity_prelude::{self as serenity, ErrorResponse}, BoxFuture};
use reqwest;
use serde_json::Value;
use chrono::{DateTime, Utc};

// Pagination function
pub async fn paginate<U: std::marker::Sync, E>(
    ctx: Context<'_>,
    pages: Vec<String>,
    refresh_data: impl Fn() -> BoxFuture<'static, Result<Vec<String>, Error>> + Send + Sync + 'static,
) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    let refresh_button_id = format!("{}refresh", ctx_id);

    // Send the initial message with buttons
    let reply = ctx
        .send(
            poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::default()
                .description(&pages[0])
                .colour(serenity::Colour::BLURPLE),
            )
            .components(vec![serenity::CreateActionRow::Buttons(vec![
                    serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                    serenity::CreateButton::new(&refresh_button_id).emoji('🔄'),
                    serenity::CreateButton::new(&next_button_id).emoji('▶'),
            ])]),
        )
        .await?;

    let message_id = reply.message().await?.id;
    let mut current_page = 0;
    let mut current_pages = pages.clone(); // Make a mutable copy of pages

    // Interaction collector with a 30-minute timeout
    if let Err(_timeout) = tokio::time::timeout(
        std::time::Duration::from_secs(3600),
        async {
            while let Some(press) = serenity::ComponentInteractionCollector::new(&ctx)
                .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
                    .await // Waits for one interaction at a time
                    {
                        match press.data.custom_id.as_str() {
                            id if id == next_button_id => {
                                current_page = (current_page + 1) % current_pages.len();
                                press
                                    .create_response(
                                        ctx.serenity_context(),
                                        serenity::CreateInteractionResponse::UpdateMessage(
                                            serenity::CreateInteractionResponseMessage::new().embed(
                                                serenity::CreateEmbed::new()
                                                .description(&current_pages[current_page])
                                                .colour(serenity::Colour::BLURPLE),
                                            ),
                                        ),
                                    )
                                    .await?;
                            }
                            id if id == prev_button_id => {
                                current_page = current_page.checked_sub(1).unwrap_or(current_pages.len() - 1);
                                press
                                    .create_response(
                                        ctx.serenity_context(),
                                        serenity::CreateInteractionResponse::UpdateMessage(
                                            serenity::CreateInteractionResponseMessage::new().embed(
                                                serenity::CreateEmbed::new()
                                                .description(&current_pages[current_page])
                                                .colour(serenity::Colour::BLURPLE),
                                            ),
                                        ),
                                    )
                                    .await?;
                            }
                            id if id == refresh_button_id => {
                                press.defer(&ctx.serenity_context()).await?; // Defer the interaction

                                match refresh_data().await {
                                    Ok(new_pages) => {
                                        current_pages = new_pages;
                                        current_page = 0;

                                        // Update the message with new pages
                                        reply
                                            .edit(
                                                ctx,
                                                poise::CreateReply::default()
                                                .embed(
                                                    serenity::CreateEmbed::new()
                                                    .description(&current_pages[current_page])
                                                    .colour(serenity::Colour::DARK_GREEN),
                                                )
                                                .components(vec![serenity::CreateActionRow::Buttons(vec![
                                                        serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                                                        serenity::CreateButton::new(&refresh_button_id).emoji('🔄'),
                                                        serenity::CreateButton::new(&next_button_id).emoji('▶'),
                                                ])]),
                                            )
                                            .await?;
                                        }
                                    Err(e) => {
                                        ctx.say(format!("Error refreshing data: {}", e)).await?;
                                    }
                                }
                            }
                            _ => continue,
                        }
                    }
            Ok::<(), serenity::Error>(())
        },
    )
    .await
    {
        // The timeout expires after 30 minutes
        ctx.channel_id()
            .delete_message(ctx, message_id)
            .await
            .unwrap_or_else(|err| {
                println!("[WARN] Failed to delete message: {}", err);
            });
    }

    Ok(())
}

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

// Function to generate pages from data
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
                "**Mythic+ Leaderboard - Page {}/{}**\n\n{}\n\n**Total Runs:** {}\n**Last Reset:** {}",
                page_idx + 1,
                (leaderboard.len() + 7) / 8,
                entries,
                total_runs,
                formatted_date
            )
        })
        .collect())
}

// Get this week's keystone completion leaderboard
#[poise::command(slash_command,)]
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

    paginate::<(), ErrorResponse>(ctx, initial_pages, refresh_closure).await?;
    Ok(())
}


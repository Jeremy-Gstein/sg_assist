use crate::{Context, Error};
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use serenity::ErrorResponse;
use poise::BoxFuture;
use serde_json::Value;
use redis::Commands;

type Leaderboard = Vec<(String, usize)>;

/// Paginated leaderboard with buttons
pub async fn paginate<U: std::marker::Sync, E>(
    ctx: Context<'_>,
    pages: Vec<String>,
    refresh_data: impl Fn() -> BoxFuture<'static, Result<Vec<String>, Error>> + Send + Sync + 'static,
) -> Result<(), serenity::Error> {
    if pages.is_empty() {
        ctx.say("No data to display.").await?;
        return Ok(());
    }

    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    let refresh_button_id = format!("{}refresh", ctx_id);

    let sent = ctx
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

    let message = sent.message().await?;
    let mut current_page = 0;
    let mut current_pages = pages;

    if let Err(_timeout) = tokio::time::timeout(
        std::time::Duration::from_secs(86400),
        async {
            while let Some(press) = serenity::ComponentInteractionCollector::new(&ctx.serenity_context())
                .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
                .await
            {
                let custom_id = press.data.custom_id.as_str();

                if custom_id == next_button_id {
                    current_page = (current_page + 1) % current_pages.len();
                } else if custom_id == prev_button_id {
                    current_page = current_page.checked_sub(1).unwrap_or(current_pages.len() - 1);
                } else if custom_id == refresh_button_id {
                    if let Err(e) = press.defer(&ctx.serenity_context().http).await {
                        eprintln!("Failed to defer interaction: {}", e);
                        continue;
                    }

                    match refresh_data().await {
                        Ok(new_pages) => {
                            if new_pages.is_empty() {
                                if let Err(e) = ctx.say("No updated data available.").await {
                                    eprintln!("Failed to send fallback message: {}", e);
                                }
                                continue;
                            }

                            current_pages = new_pages;
                            current_page = 0;

                            if let Err(e) = message
                                .edit(
                                    &ctx.serenity_context().http,
                                    serenity::EditMessage::new()
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
                                .await
                            {
                                eprintln!("Failed to edit message after refresh: {}", e);
                            }

                            continue;
                        }
                        Err(e) => {
                            if let Err(err) = ctx.say(format!("Error refreshing data: {}", e)).await {
                                eprintln!("Failed to send error message: {}", err);
                            }
                            continue;
                        }
                    }
                }

                if let Err(e) = press
                    .create_response(
                        &ctx.serenity_context().http,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new().embed(
                                serenity::CreateEmbed::new()
                                    .description(&current_pages[current_page])
                                    .colour(serenity::Colour::BLURPLE),
                            ),
                        ),
                    )
                    .await
                {
                    eprintln!("Failed to update interaction response: {}", e);
                }
            }

            Ok::<(), serenity::Error>(())
        },
    )
    .await
    {
        if let Err(e) = ctx
            .channel_id()
            .delete_message(&ctx.serenity_context().http, message.id)
            .await
        {
            eprintln!("[WARN] Failed to delete message after timeout: {}", e);
        }
    }

    Ok(())
}

/// Generate paginated leaderboard pages
pub fn generate_pages_from_leaderboard(
    leaderboard: &Leaderboard,
    period_info: &Value,
) -> Result<Vec<String>, Error> {
    let date_str = period_info["periods"][0]["current"]["start"]
        .as_str()
        .ok_or("Missing period start date")?;
    let datetime = DateTime::parse_from_rfc3339(date_str)?.with_timezone(&Utc);
    let unix_timestamp = datetime.timestamp();
    let formatted_date = format!("<t:{}:R>", unix_timestamp);

    let total_runs: usize = leaderboard.iter().map(|(_, count)| *count).sum();

    let mut sorted = leaderboard.clone();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(sorted
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
                (sorted.len() + 7) / 8,
                entries,
                total_runs,
                formatted_date
            )
        })
        .collect())
}

/// Fetch the current period ID from Raider.IO API
pub async fn fetch_period_id() -> Result<Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://raider.io/api/v1/periods")
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(response)
}

/// Read the leaderboard from Redis
async fn read_leaderboard() -> redis::RedisResult<Leaderboard> {
    let client = redis::Client::open("redis://:sgdbadmin@redis/").unwrap();
    let mut conn = client.get_connection().unwrap();
    let vec_leaderboard: Vec<(String, usize)> = conn.hgetall("leaderboard")?;
    Ok(vec_leaderboard)
}

/// Show the guilds mythic+ leaderboard for current week.
#[poise::command(slash_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let period_info = fetch_period_id().await?;
    let leaderboard = read_leaderboard().await?;
    let initial_pages = generate_pages_from_leaderboard(&leaderboard, &period_info)?;

    let refresh_closure = {
        let http = ctx.serenity_context().http.clone();
        move || {
            let _http = http.clone();
            Box::pin(async move {
                let new_period = fetch_period_id().await?;
                let new_board = read_leaderboard().await?;
                generate_pages_from_leaderboard(&new_board, &new_period)
            }) as BoxFuture<'static, Result<Vec<String>, Error>>
        }
    };

    paginate::<(), ErrorResponse>(ctx, initial_pages, refresh_closure).await?;
    Ok(())
}

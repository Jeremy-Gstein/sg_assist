use crate::{Context, Error};
use crate::config;
use poise::serenity_prelude as serenity;
use poise::BoxFuture;
use serde_json::Value;

type Leaderboard = Vec<(String, usize)>;

// ---------------------------------------------------------------------------
// API helpers
// ---------------------------------------------------------------------------

/// Fetch current-week historical data from wowaudit.
async fn fetch_historical_data() -> Result<Value, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .get("https://www.wowaudit.com/v1/historical_data")
        .header("Authorization", config::wowaudit_token())
        .send()
        .await?
        .json::<Value>()
        .await
}

/// Build a sorted leaderboard (name, keys_done) from the wowaudit response.
/// Mirrors the bash script: sort_by(.data.dungeons_done | length) | reverse | .[:10]
fn build_leaderboard(data: &Value) -> Leaderboard {
    let Some(characters) = data["characters"].as_array() else {
        return vec![];
    };

    let mut entries: Vec<(String, usize)> = characters
        .iter()
        .filter_map(|c| {
            let name = c["name"].as_str()?.to_string();
            let count = c["data"]["dungeons_done"]
                .as_array()
                .map(|v| v.len())
                .unwrap_or(0);
            Some((name, count))
        })
        .collect();

    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries
}

// ---------------------------------------------------------------------------
// Page generation
// ---------------------------------------------------------------------------

fn generate_pages(leaderboard: &Leaderboard) -> Vec<String> {
    if leaderboard.is_empty() {
        return vec!["No data available for this week.".to_string()];
    }

    let total_runs: usize = leaderboard.iter().map(|(_, c)| c).sum();

    leaderboard
        .chunks(8)
        .enumerate()
        .map(|(page_idx, chunk)| {
            let total_pages = (leaderboard.len() + 7) / 8;
            let entries = chunk
                .iter()
                .map(|(name, count)| format!("- **{}**: {} keys completed", name, count))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "**Mythic+ Leaderboard — Page {}/{}**\n\n{}\n\n**Total Runs This Week:** {}",
                page_idx + 1,
                total_pages,
                entries,
                total_runs,
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Paginated embed with ◀ 🔄 ▶ buttons
// ---------------------------------------------------------------------------

async fn paginate(
    ctx: Context<'_>,
    pages: Vec<String>,
    refresh_data: impl Fn() -> BoxFuture<'static, Result<Vec<String>, Error>> + Send + Sync + 'static,
) -> Result<(), serenity::Error> {
    if pages.is_empty() {
        ctx.say("No data to display.").await?;
        return Ok(());
    }

    let ctx_id = ctx.id();
    let prev_id = format!("{}prev", ctx_id);
    let next_id = format!("{}next", ctx_id);
    let refresh_id = format!("{}refresh", ctx_id);
    let delete_id = format!("{}delete", ctx_id);

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .embed(
                    serenity::CreateEmbed::default()
                        .description(&pages[0])
                        .colour(serenity::Colour::BLURPLE),
                )
                .components(vec![serenity::CreateActionRow::Buttons(vec![
                    serenity::CreateButton::new(&prev_id).emoji('◀'),
                    serenity::CreateButton::new(&next_id).emoji('▶'),
                    serenity::CreateButton::new(&refresh_id).emoji('🔄').style(serenity::ButtonStyle::Success),
                    serenity::CreateButton::new(&delete_id).emoji('🗑').style(serenity::ButtonStyle::Danger),
                ])]),
        )
        .await?;

    let message_id = reply.message().await?.id;
    let mut current_page = 0usize;
    let mut current_pages = pages;

    let timed_out = tokio::time::timeout(
        std::time::Duration::from_secs(86400),
        async {
            while let Some(press) = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
                .filter(move |p| p.data.custom_id.starts_with(&ctx_id.to_string()))
                .await
            {
                let id = press.data.custom_id.clone();

                if id == refresh_id {
                    if let Err(e) = press.defer(&ctx.serenity_context().http).await {
                        eprintln!("Failed to defer refresh: {}", e);
                        continue;
                    }
                    match refresh_data().await {
                        Ok(new_pages) if !new_pages.is_empty() => {
                            current_pages = new_pages;
                            current_page = 0;
                            let _ = ctx
                                .channel_id()
                                .edit_message(
                                    &ctx.serenity_context().http,
                                    message_id,
                                    serenity::EditMessage::new()
                                        .embed(
                                            serenity::CreateEmbed::new()
                                                .description(&current_pages[0])
                                                .colour(serenity::Colour::DARK_GREEN),
                                        )
                                        .components(vec![serenity::CreateActionRow::Buttons(vec![
                                            serenity::CreateButton::new(&prev_id).emoji('◀'),
                                            serenity::CreateButton::new(&next_id).emoji('▶'),
                                            serenity::CreateButton::new(&refresh_id).emoji('🔄').style(serenity::ButtonStyle::Success),
                                            serenity::CreateButton::new(&delete_id).emoji('🗑').style(serenity::ButtonStyle::Danger),
                                        ])]),
                                )
                                .await;
                        }
                        Ok(_) => { let _ = ctx.say("No updated data available.").await; }
                        Err(e) => { let _ = ctx.say(format!("Error refreshing: {}", e)).await; }
                    }
                    continue;
                }

                if id == delete_id {
                    let _ = press.defer(&ctx.serenity_context().http).await;
                    let _ = ctx
                        .channel_id()
                        .delete_message(&ctx.serenity_context().http, message_id)
                        .await;
                    return Ok(());
                }

                if id == next_id {
                    current_page = (current_page + 1) % current_pages.len();
                } else if id == prev_id {
                    current_page = current_page.checked_sub(1).unwrap_or(current_pages.len() - 1);
                }

                let _ = press
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
                    .await;
            }
            Ok::<(), serenity::Error>(())
        },
    )
    .await;

    // Remove buttons after 24h timeout
    if timed_out.is_err() {
        let _ = ctx
            .channel_id()
            .edit_message(
                &ctx.serenity_context().http,
                message_id,
                serenity::EditMessage::new().components(vec![]),
            )
            .await;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Slash command
// ---------------------------------------------------------------------------

/// Show the guild's Mythic+ leaderboard for the current week.
#[poise::command(slash_command)]
pub async fn keysdone(ctx: Context<'_>) -> Result<(), Error> {
    let data = match fetch_historical_data().await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to fetch wowaudit data: {}", e);
            ctx.send(poise::CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("Error")
                    .description("Could not reach the WowAudit API. Please try again later.")
                    .colour(serenity::Colour::RED)
                )
            ).await?;
            return Ok(());
        }
    };
    let leaderboard = build_leaderboard(&data);
    let pages = generate_pages(&leaderboard);

    let refresh_closure = move || {
        Box::pin(async move {
            let data = fetch_historical_data().await?;
            let lb = build_leaderboard(&data);
            Ok(generate_pages(&lb))
        }) as BoxFuture<'static, Result<Vec<String>, Error>>
    };

    paginate(ctx, pages, refresh_closure).await?;
    Ok(())
}

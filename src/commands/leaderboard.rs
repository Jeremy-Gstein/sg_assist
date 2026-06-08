// src/commands/leaderboard.rs
//
// Fetches the weekly Mythic+ leaderboard from the mplus-tracker API.
// Uses GET /leaderboard?scope=week which returns:
//   - Logical players (alts aggregated under their label)
//   - Untracked characters (guild members not assigned to a player, shown by name)
// Both are sorted by key count descending by the server.

use crate::{Context, Error, config};
use poise::serenity_prelude as serenity;
use poise::BoxFuture;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

fn auth_client() -> reqwest::Result<reqwest::Client> {
    let token = format!("Bearer {}", config::api_token());
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&token).expect("API token contains invalid header characters"),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
}

// ---------------------------------------------------------------------------
// API types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone)]
struct LeaderboardEntry {
    display_name: String,
    count:        i64,
    is_player:    bool,
}

#[derive(Deserialize)]
struct LeaderboardResponse {
    from:    String,
    entries: Vec<LeaderboardEntry>,
}

// ---------------------------------------------------------------------------
// Fetch
// ---------------------------------------------------------------------------

async fn fetch_leaderboard() -> Result<(Vec<LeaderboardEntry>, String), Error> {
    let client = auth_client().map_err(|e| format!("Failed to build HTTP client: {e}"))?;
    let url = format!("{}/leaderboard?scope=week", config::tracker_url());

    let resp: LeaderboardResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Could not reach mplus-tracker /leaderboard: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse leaderboard response: {e}"))?;

    Ok((resp.entries, resp.from))
}

// ---------------------------------------------------------------------------
// Format
// ---------------------------------------------------------------------------

fn format_week_start(iso: &str) -> String {
    use chrono::{DateTime, Utc};
    use chrono_tz::America::New_York;

    DateTime::parse_from_rfc3339(iso)
        .ok()
        .map(|dt| {
            dt.with_timezone(&Utc)
                .with_timezone(&New_York)
                .format("%-I:%M %p ET, %a %b %-d")
                .to_string()
        })
        .unwrap_or_else(|| iso.to_string())
}

fn generate_pages(entries: &[LeaderboardEntry], week_from: &str) -> Vec<String> {
    if entries.is_empty() {
        return vec![
            "No data available for this week. Run `/update/all` on the tracker first.".to_string(),
        ];
    }

    let total_runs: i64 = entries.iter().map(|e| e.count).sum();
    let week_label = if week_from.is_empty() {
        "this week".to_string()
    } else {
        format!("since {}", format_week_start(week_from))
    };

    let total_pages = entries.len().div_ceil(8);

    entries
        .chunks(8)
        .enumerate()
        .map(|(page_idx, chunk)| {
            let start_rank = page_idx * 8 + 1;
            let lines = chunk
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let rank = start_rank + i;
                    let medal = match rank {
                        1 => "🥇",
                        2 => "🥈",
                        3 => "🥉",
                        _ => "▫️",
                    };
                    // Untracked characters get a subtle indicator so people
                    // know they're a raw character, not an aggregated player.
                    let suffix = if entry.is_player { "" } else { " *(char)*" };
                    format!(
                        "{medal} **{}**{} — {} key{}",
                        entry.display_name,
                        suffix,
                        entry.count,
                        if entry.count == 1 { "" } else { "s" }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "**Mythic+ Leaderboard — Page {}/{}**\n_{}_\n\n{}\n\n**Guild Total:** {} key{}",
                page_idx + 1,
                total_pages,
                week_label,
                lines,
                total_runs,
                if total_runs == 1 { "" } else { "s" },
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pagination (same ◀ 🔄 ▶ 🗑 pattern)
// ---------------------------------------------------------------------------

async fn paginate(
    ctx: Context<'_>,
    pages: Vec<String>,
    refresh_data: impl Fn() -> BoxFuture<'static, Result<Vec<String>, Error>>
        + Send
        + Sync
        + 'static,
) -> Result<(), Error> {
    if pages.is_empty() {
        ctx.say("No data to display.").await?;
        return Ok(());
    }

    let ctx_id     = ctx.id();
    let prev_id    = format!("{}prev",    ctx_id);
    let next_id    = format!("{}next",    ctx_id);
    let refresh_id = format!("{}refresh", ctx_id);
    let delete_id  = format!("{}delete",  ctx_id);

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
                    serenity::CreateButton::new(&refresh_id)
                        .emoji('🔄')
                        .style(serenity::ButtonStyle::Success),
                    serenity::CreateButton::new(&delete_id)
                        .emoji('🗑')
                        .style(serenity::ButtonStyle::Danger),
                ])]),
        )
        .await?;

    let message_id     = reply.message().await?.id;
    let mut cur_page   = 0usize;
    let mut cur_pages  = pages;

    let timed_out = tokio::time::timeout(
        std::time::Duration::from_secs(86400),
        async {
            while let Some(press) =
                serenity::ComponentInteractionCollector::new(ctx.serenity_context())
                    .filter(move |p| p.data.custom_id.starts_with(&ctx_id.to_string()))
                    .await
            {
                let id = press.data.custom_id.clone();

                if id == refresh_id {
                    if let Err(e) = press.defer(&ctx.serenity_context().http).await {
                        eprintln!("Failed to defer refresh: {e}");
                        continue;
                    }
                    match refresh_data().await {
                        Ok(new) if !new.is_empty() => {
                            cur_pages = new;
                            cur_page  = 0;
                            let _ = ctx.channel_id().edit_message(
                                &ctx.serenity_context().http,
                                message_id,
                                serenity::EditMessage::new()
                                    .embed(serenity::CreateEmbed::new()
                                        .description(&cur_pages[0])
                                        .colour(serenity::Colour::DARK_GREEN))
                                    .components(vec![serenity::CreateActionRow::Buttons(vec![
                                        serenity::CreateButton::new(&prev_id).emoji('◀'),
                                        serenity::CreateButton::new(&next_id).emoji('▶'),
                                        serenity::CreateButton::new(&refresh_id).emoji('🔄').style(serenity::ButtonStyle::Success),
                                        serenity::CreateButton::new(&delete_id).emoji('🗑').style(serenity::ButtonStyle::Danger),
                                    ])]),
                            ).await;
                        }
                        Ok(_)    => { let _ = ctx.say("No updated data available.").await; }
                        Err(e)   => { let _ = ctx.say(format!("Error refreshing: {e}")).await; }
                    }
                    continue;
                }

                if id == delete_id {
                    let _ = press.defer(&ctx.serenity_context().http).await;
                    let _ = ctx.channel_id()
                        .delete_message(&ctx.serenity_context().http, message_id)
                        .await;
                    return Ok(());
                }

                if id == next_id {
                    cur_page = (cur_page + 1) % cur_pages.len();
                } else if id == prev_id {
                    cur_page = cur_page.checked_sub(1).unwrap_or(cur_pages.len() - 1);
                }

                let _ = press.create_response(
                    &ctx.serenity_context().http,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new().embed(
                            serenity::CreateEmbed::new()
                                .description(&cur_pages[cur_page])
                                .colour(serenity::Colour::BLURPLE),
                        ),
                    ),
                ).await;
            }
            Ok::<(), Error>(())
        },
    ).await;

    if timed_out.is_err() {
        let _ = ctx.channel_id().edit_message(
            &ctx.serenity_context().http,
            message_id,
            serenity::EditMessage::new().components(vec![]),
        ).await;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Slash command
// ---------------------------------------------------------------------------

/// Show the guild's Mythic+ leaderboard for the current weekly reset.
#[poise::command(slash_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let (entries, week_from) = match fetch_leaderboard().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to fetch leaderboard: {e}");
            ctx.send(
                poise::CreateReply::default().embed(
                    serenity::CreateEmbed::new()
                        .title("Error")
                        .description("Could not reach the mplus-tracker API. Please try again later.")
                        .colour(serenity::Colour::RED),
                ),
            ).await?;
            return Ok(());
        }
    };

    let pages = generate_pages(&entries, &week_from);

    let refresh_closure = move || {
        Box::pin(async move {
            let (e, wf) = fetch_leaderboard().await?;
            Ok(generate_pages(&e, &wf))
        }) as BoxFuture<'static, Result<Vec<String>, Error>>
    };

    paginate(ctx, pages, refresh_closure).await?;
    Ok(())
}

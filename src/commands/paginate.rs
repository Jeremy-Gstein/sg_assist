use poise::serenity_prelude as serenity;
use poise::Context;
use crate::Error;
use poise::BoxFuture;

pub async fn paginate<U: std::marker::Sync, E>(
    ctx: Context<'_, U, E>,
    pages: Vec<String>,
    refresh_data: impl Fn() -> BoxFuture<'static, Result<Vec<String>, Error>> + Send + Sync + 'static,
) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    let refresh_button_id = format!("{}refresh", ctx_id);

    // Send initial message with buttons
    let reply = ctx.send(
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
    ).await?;

    let message_id = reply.message().await?.id;
    let mut current_page = 0;
    let mut current_pages = pages.clone();  // Make a mutable copy of the pages

    while let Some(press) = serenity::ComponentInteractionCollector::new(&ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(60 * 60)) // 1 hour timeout
        .await
    {
        match press.data.custom_id.as_str() {
            // If "next" button pressed, change to next page
            id if id == next_button_id => {
                current_page = (current_page + 1) % current_pages.len();
                press.create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(
                                serenity::CreateEmbed::new()
                                    .description(&current_pages[current_page])
                                    .colour(serenity::Colour::BLURPLE),
                            ),
                    ),
                ).await?;
            }
            // If "previous" button pressed, change to previous page
            id if id == prev_button_id => {
                current_page = current_page.checked_sub(1).unwrap_or(current_pages.len() - 1);
                press.create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(
                                serenity::CreateEmbed::new()
                                    .description(&current_pages[current_page])
                                    .colour(serenity::Colour::BLURPLE),
                            ),
                    ),
                ).await?;
            }
            // If "refresh" button pressed, refresh the data
            id if id == refresh_button_id => {
                press.defer(&ctx.serenity_context()).await?;  // Defer the interaction

                // Call the refresh function to get new data
                match refresh_data().await {
                    Ok(new_pages) => {
                        current_pages = new_pages;
                        current_page = current_page.min(current_pages.len().saturating_sub(1));

                        // Update message with new pages
                        reply.edit(
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
                                ])])
                        ).await?;
                    }
                    Err(e) => {
                        ctx.say(format!("Error refreshing data: {}", e)).await?;
                    }
                }
            }
            _ => continue,
        }
    }

    // Background task to auto-delete the message after timeout (optional)
    let serenity_http = ctx.serenity_context().http.clone();  // Clone the serenity HTTP client
    let channel_id = ctx.channel_id();  // Clone channel ID

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
        if let Err(e) = channel_id.delete_message(&serenity_http, message_id).await {
            println!("[WARN] Failed to delete message after timeout: {}", e);
        }
    });

    Ok(())
}


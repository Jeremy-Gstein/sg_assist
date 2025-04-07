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


    let timout_seconds: u64 = 60 * 60 * 4; //seconds * min * hour

    // Interaction collector with a 30-minute timeout
    if let Err(_timeout) = tokio::time::timeout(
        std::time::Duration::from_secs(timout_seconds),
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
                                current_page = current_page.min(current_pages.len().saturating_sub(1));

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
                                                serenity::CreateButton::new(&refresh_button_id)
                                                    .emoji('🔄'),
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


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

    let mut current_pages = pages.clone();
    let mut current_page = 0;

    // Send initial message with buttons
    let reply = ctx
        .send(
            poise::CreateReply::default()
                .embed(
                    serenity::CreateEmbed::new()
                        .description(&current_pages[current_page])
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

    // Interaction Collector
    let timeout_duration = std::time::Duration::from_secs(3600); //60 * 30); // 30 minutes
    if let Err(_) = tokio::time::timeout(timeout_duration, async {
        while let Some(press) = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
            .filter(move |press| press.message.id == message_id)
            .await
        {
            match press.data.custom_id.as_str() {
                id if id == next_button_id => {
                    current_page = (current_page + 1) % current_pages.len();
                    press.create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                            .embed(serenity::CreateEmbed::new().description(&current_pages[current_page]))
                        ),
                    ).await?;
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .description(&current_pages[current_page])
                                        .colour(serenity::Colour::BLURPLE),
                                )
                                .components(vec![serenity::CreateActionRow::Buttons(vec![
                                    serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                                    serenity::CreateButton::new(&refresh_button_id).emoji('🔄'),
                                    serenity::CreateButton::new(&next_button_id).emoji('▶'),
                                ])]),
                        )
                        .await?;
                }
                id if id == prev_button_id => {
                    current_page = if current_page == 0 {
                        current_pages.len() - 1
                    } else {
                        current_page - 1
                    };
                    // Acknowledge and update the interaction
                    press.create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                            .embed(serenity::CreateEmbed::new().description(&current_pages[current_page])),
                        ),
                    ).await?;
                    reply
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .description(&current_pages[current_page])
                                        .colour(serenity::Colour::BLURPLE),
                                )
                                .components(vec![serenity::CreateActionRow::Buttons(vec![
                                    serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                                    serenity::CreateButton::new(&refresh_button_id).emoji('🔄'),
                                    serenity::CreateButton::new(&next_button_id).emoji('▶'),
                                ])]),
                        )
                        .await?;
                }
                id if id == refresh_button_id => {
                    if press.defer(&ctx.serenity_context()).await.is_ok() {
                        match refresh_data().await {
                            Ok(new_pages) => {
                                current_pages = new_pages;
                                current_page = 0; // Reset to the first page
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
                                // Notify the user of an error during refresh
                                ctx.say(format!("Error refreshing data: {}", e)).await?;
                            }
                        }
                    }
                }
                _ => continue, // Ignore unrelated interactions
            }
        }
        Ok::<(), serenity::Error>(())
    })
    .await
    {
        // Interaction collector has timed out, delete the message (cleanup)
        ctx.channel_id()
            .delete_message(&ctx.serenity_context().http, message_id)
            .await
            .unwrap_or_else(|err| {
                println!("[WARN] Failed to delete message: {}", err);
            });
    }

    Ok(())
}

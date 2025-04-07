use poise::serenity_prelude as serenity;
use crate::Error;
use crate::Context;
use futures::future::BoxFuture;
use poise::serenity_prelude::Colour;

pub async fn paginate(
    ctx: Context<'_>,
    pages: Vec<String>,
    refresh_data: impl Fn() -> BoxFuture<'static, Result<Vec<String>, Error>> + Send + Sync + 'static,
) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    let refresh_button_id = format!("{}refresh", ctx_id);

    // Store context for async deletion task
    let serenity_ctx = ctx.serenity_context().clone();
    let channel_id = ctx.channel_id();
    let http = serenity_ctx.http.clone();

    // Send initial message
    let reply = ctx.send(
        poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::default()
                    .description(&pages[0])
                    .colour(Colour::BLURPLE),
            )
            .components(vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                serenity::CreateButton::new(&refresh_button_id).emoji('🔄'),
                serenity::CreateButton::new(&next_button_id).emoji('▶'),
            ])]),
    ).await?;

    let message_id = reply.message().await?.id;
    let mut current_page = 0;
    let mut current_pages = pages;

    while let Some(press) = serenity::ComponentInteractionCollector::new(&ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(60 * 60)) // 1 hour timeout
        .await
    {
        match press.data.custom_id.as_str() {
            id if id == next_button_id => {
                current_page = (current_page + 1) % current_pages.len();
                press.create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(
                                serenity::CreateEmbed::new()
                                    .description(&current_pages[current_page])
                                    .colour(Colour::BLURPLE),
                            ),
                    ),
                ).await?;
            }
            id if id == prev_button_id => {
                current_page = current_page.checked_sub(1).unwrap_or(current_pages.len() - 1);
                press.create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(
                                serenity::CreateEmbed::new()
                                    .description(&current_pages[current_page])
                                    .colour(Colour::BLURPLE),
                            ),
                    ),
                ).await?;
            }
            id if id == refresh_button_id => {
                // Defer the interaction first
                press.defer(&ctx.serenity_context()).await?;

                // Execute the refresh
                match refresh_data().await {
                    Ok(new_pages) => {
                        current_pages = new_pages;
                        current_page = current_page.min(current_pages.len().saturating_sub(1));
                        
                        // Edit the message
                        reply.edit(
                            ctx, 
                            poise::CreateReply::default()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .description(&current_pages[current_page])
                                        .colour(Colour::DARK_GREEN),
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
                        continue;
                    }
                }
            }
            _ => continue,
        }
    }



    // Setup auto-delete in a background task
    // we only get here after .timout()
    tokio::spawn(async move {
        // if its been 5 min since last interation.. delete it
        //tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        println!("[DEBUG] - TIMOUT for {} has been reatched. Attempting to Delete.", message_id);
        match channel_id.delete_message(&http, message_id).await {
            Ok(_) => println!("[DEBUG] - Delete task completed for: {}", message_id),
            Err(e) => println!("[WARN] - Delete task for {} failed with error: {}", message_id, e )
        }
    });

    Ok(())
}

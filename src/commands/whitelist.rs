use crate::Context;
use crate::Error;

/// Add a Minecraft account to mc.seeemsgood.org whitelist.
#[poise::command(slash_command, broadcast_typing, ephemeral)]
pub async fn whitelist(
    ctx: Context<'_>,
    #[description = "Player name to whitelist"] name: String,
) -> Result<(), Error> {
    use rcon::Connection;
    use poise::serenity_prelude::CreateEmbed;

    // Connect to the Minecraft server via RCON
    let mut conn = Connection::connect("server-ip:25575", "YourStrongPassword").await?;

    // Send the whitelist add command
    let resp: String = conn.cmd(&format!("whitelist add {}", name)).await?;

    // Build a nice embed depending on response
    let (title, description, colour) = if resp.to_lowercase().contains("added") {
        (
            "Whitelist Update".to_string(),
            format!("✅ Successfully added **{}** to the whitelist.\n```\n{}```", name, resp),
            poise::serenity_prelude::Colour::from_rgb(67, 181, 129), // green
        )
    } else {
        (
            "Error".to_string(),
            format!("❌ Could not add **{}**.\n```\n{}```", name, resp),
            poise::serenity_prelude::Colour::from_rgb(240, 71, 71), // red
        )
    };

    // Send the embed to Discord
    ctx.send(
        poise::CreateReply::default().embed(
            CreateEmbed::new()
                .title(title)
                .description(description)
                .colour(colour),
        ),
    )
    .await?;

    Ok(())
}


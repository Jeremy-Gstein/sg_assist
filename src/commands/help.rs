use crate::Context;
use crate::Error;

/// How to use bot with examples.
#[poise::command(slash_command, broadcast_typing)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    // Build the embed without mutability
    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title("SG Assistant Help")
        .description("Here is a list of commands and their descriptions:")
        .field("`/help`", "List of commands. If you need help, message or ping `@.shodo`.", false)
        .field("`/updatesim`", "Update your character's WoWAudit RCLootCouncil Wishlist.", false)
        .field("`/roster`", "List the current WoWAudit roster.", false)
        .field("`/keysdone`", "Returns a list of the top 8 guildies for the current week's reset with the number of keys completed.", false)
        .field("`/mykeys`", "Input your in-game character's name to see this week's Mythic+ progress. Check the roster to ensure your character is listed.", false)
        .field("`/vault`", "Input your in-game character's name to see your current vault progress for the week.", false)
        .colour(poise::serenity_prelude::Colour::from_rgb(114, 137, 218)); 

    // Send the embed without cloning
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}


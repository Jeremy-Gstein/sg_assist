use crate::Context;
use crate::Error;


/// How to use bot with examples. 
#[poise::command(interaction_context = "Guild|BotDm|PrivateChannel", slash_command, broadcast_typing)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let help = "`/help` - list of commands. if you need any help message or ping `@.shodo`";
    let updatesim_help = "`/updatesim` - Update your characters Wowaudit RCLootcouncil Wishlist.";
    let roster_help = "`/roster` - list the current wowaudit roster.";
    let keysdone_help = "`/keysdone` - Returns a list of the top 8 guildies for the current weeks reset with the number of keys completed.";
    let mykeys_help = "`/mykeys` - Input your ingame characters name to see this weeks mythic+ progress. check the roster to make sure your character is on the list!"; 
    let vault_help = "`/vault` - Input your ingame characters name to see your current vault progress for the week.";
    let jackpot_help = "`/jackpot` and `/megajackpot` - not currenty running";
    let msg_send = &format!("### Thanks for using SG Assistant!!\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n", &help, &updatesim_help, &roster_help, &mykeys_help, &keysdone_help, &vault_help, &jackpot_help);
    ctx.say(msg_send).await?;
    Ok(())
}

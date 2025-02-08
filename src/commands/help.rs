use crate::Context;
use crate::Error;


/// How to use bot with examples. 
#[poise::command(slash_command, broadcast_typing)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let help = "`/help` - list of commands. if you need any help message or ping `@.shodo` :shodoheart: ";
    let keysdone_help = "`/keysdone` - Returns a list of the top 8 guildies for the current weeks reset with the number of keys completed.";
    let vautlt_help = "`/vault` - Input your ingame characters name to see your current weeks vauly progress";
    let jackpot_help = "`/jackpot` and `/megajackpot` - not currenty running";
    let msg_send = &format!("### Thanks for using SG Assistant!!\n{}\n{}\n{}\n{}\n", &help, &keysdone_help, &vautlt_help, &jackpot_help);
    ctx.say(msg_send).await?;
    Ok(())
}

use crate::Context; // Use the Context type alias from main.rs
use crate::Error; // Use the Error type alias from main.rs

/// general information about megajackpot
#[poise::command(slash_command, broadcast_typing)]
pub async fn megajackpot(ctx: Context<'_>) -> Result<(), Error> {
    let header = format!("## Chint's [MegaGewdOver900Jackpot!](https://docs.google.com/spreadsheets/d/1fVPBtzSeZrGlUYHrirVTeYqNZWE4EHlDhRNOf4SADu4/edit?usp=sharing)");
    let pin = format!("https://discord.com/channels/1154218486088859659/1154218488760639529/1272697779004768329\n");
    let footer = format!("**TLDR:**\n- Send gold you want to enter with to the character `Sgbankteller` on Stormrage Alliance.\n- After the gold is collected from the mail, your entries will be added to the list.\n- The winner will be announced on a stream in Discord.");
    let response = format!("{} {} ### {}", header, pin, footer);
    ctx.say(response).await?;
    Ok(())
}

use poise::Modal;
use reqwest;
use serde_json::json;
use crate::Data;
use crate::Error;
type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;


// Modal struct to collect inputs
#[derive(Debug, Default, Modal)]
#[name = "Submit Droptimizer"]
struct UpdateSimModal {
    #[name  = "Submit your Raidbots Droptimizer to Wowaudit"]
    #[placeholder = "Example: https://www.raidbots.com/simbot/report/vqBr9"]
    id: String,
}

async fn update_wishlist(id: &str) -> Result<serde_json::Value, reqwest::Error> {
    let token = std::env::var("WOWAUDIT_TOKEN").expect("missing WOWAUDIT_TOKEN");
    let client = reqwest::Client::new();
    let payload = json!({
        "report_id": id,
        "configuration_name": "Single Target",
        "replace_manual_edits": true,
        "clear_conduits": true
    });
    let response = client
        .post("https://wowaudit.com/v1/wishlists")
        .header("accept", "application/json")
        .header("Authorization", token)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    Ok(response)
}

fn extract_id(input: &str) -> Option<&str> {
    if !input.contains('/') {
        return Some(input);
    }
    input.split('/').last()
}

<<<<<<< HEAD
/// Update wowaudit raidbot sim for RClootcouncil. 
#[poise::command(interaction_context = "Guild|BotDm|PrivateChannel", slash_command, broadcast_typing)]
pub async fn updatesim(
    ctx: Context<'_>,
    #[description = "Character name on roster"] name: String, 
    #[description = "Raidbots ID or full URL"] id: String, 
) -> Result<(), Error> {
    let mut msg_send = String::new();
    let id = extract_id(&id).ok_or_else(|| Error::from("Invalid ID or URL"))?;
    match update_wishlist(id, &name).await {
        Ok(send_wishlist) => {
            msg_send.push_str(&format!("Status: {} for {}'s report {}.", send_wishlist, name, id));
        },
        Err(e) => {
            msg_send.push_str(&format!("Error formatting: {}", e));
=======
/// Update wowaudit raidbot sim for RClootcouncil.
#[poise::command(slash_command, broadcast_typing, ephemeral)]
pub async fn updatesim(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    // Execute modal to get user input
    let modal_data = poise::execute_modal(
        ctx,
        Some(UpdateSimModal::default()),
        None,
    )
    .await?;

    if let Some(data) = modal_data {
        let id = extract_id(&data.id).ok_or_else(|| Error::from("Invalid ID or URL"))?;
        match update_wishlist(id).await {
            
            Ok(send_wishlist) => {
                if send_wishlist["created"] == true {
                    ctx.say("Finished updating Droptimizer")
                    .await?;
                } else {
                    ctx.say(format!("Error: {}.. Please Try Again", send_wishlist["base"]))
                    .await?;
                }
            },
            Err(e) => {
                ctx.say(format!("Error formatting: {}", e)).await?;
            }
>>>>>>> ed35067c231c77e8bd00490103c0551eaff4bfe8
        }
    } else {
        ctx.say("Modal input was cancelled or not provided.").await?;
    }

    Ok(())
}

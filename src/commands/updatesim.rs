use reqwest;
use serde_json::json;
use crate::Context;
use crate::Error;

async fn update_wishlist(id: &str, name: &str) -> Result<serde_json::Value, reqwest::Error> {
    let token = std::env::var("WOWAUDIT_TOKEN").expect("missing WOWAUDIT_TOKEN");
    let client = reqwest::Client::new();
    let payload = json!({
        "report_id": id,
        "character_name": name,
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
    // if input dosent have a / assume its been formated
    if !input.contains('/') {
        return Some(input);
    }
    // return end of raidbots link.
    input.split('/').last()
}

/// Update wowaudit raidbot sim for RClootcouncil. 
#[poise::command(slash_command, broadcast_typing)]
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
        }
    }
    ctx.say(msg_send).await?;
    Ok(())
}

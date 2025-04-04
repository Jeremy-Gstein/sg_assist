use serde_json::Value;
use reqwest;
use crate::Context;
use crate::Error;


async fn fetch_roster_data() -> Result<Value, reqwest::Error> {
    let token = std::env::var("WOWAUDIT_TOKEN").expect("missing WOWAUDIT_TOKEN");
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.wowaudit.com/v1/characters")
        .header("Authorization", token)
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(response)
}

#[poise::command(slash_command, broadcast_typing, ephemeral)]
pub async fn roster(ctx: Context<'_>) -> Result<(), Error> {
    let response = fetch_roster_data().await?;
    let mut msg_send = String::new();

    msg_send.push_str("## Wowaudit Roster\n");

    if let Some(characters) = response.as_array() {
        let mut names: Vec<String> = characters
            .iter()
            .filter_map(|character| {
                character["name"].as_str().map(String::from)
            })
            .collect();

        names.sort(); // Sort names alphabetically
        let mut index = 0;
        for (i, name) in names.iter().enumerate() {
            if i > 0 && index == 0 {
                msg_send.push_str("\n");
            }
            msg_send.push_str(name);
            
            if i < names.len() - 1 {
                msg_send.push_str(", ");
            }
            
            index += 1;
            if index >= 5 {
                index = 0;
            }
        }
    } else {
        msg_send = "No roster data available.".to_string();
    }

    ctx.say(msg_send).await?;

    Ok(())
}

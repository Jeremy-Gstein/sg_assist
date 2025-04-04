use reqwest;
use serde_json::Value;
use crate::Context;
use crate::Error;

pub async fn fetch_character_data() -> Result<Value, reqwest::Error> {
    let token = std::env::var("WOWAUDIT_TOKEN").expect("missing WOWAUDIT_TOKEN");
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.wowaudit.com/v1/historical_data")
        .header("Authorization", token)
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(response)
}

/// Search for a character's weekly vault status (must be on the wowaudit roster)
#[poise::command(slash_command, broadcast_typing, ephemeral)]
pub async fn vault(
    ctx: Context<'_>,
    #[description = "Character name to search for"] name: String,
) -> Result<(), Error> {
    let response = fetch_character_data().await?;
    let mut msg_send = String::new();

    if let Some(characters) = response["characters"].as_array() {
        let mut character_found = false;
        for character in characters {
            let char_name = character["name"].as_str().unwrap_or("Unknown");
            
            if char_name.to_lowercase() != name.to_lowercase() {
                continue;
            }

            character_found = true;
            msg_send.push_str(&format!("### ~ {}'s Weekly Great Vault Progress ~\n", char_name));
            
            if let Some(vault_options) = character["data"]["vault_options"].as_object() {
                for (category, options) in vault_options {
                    if category == "world" {
                        continue; // Skip the "World" category
                    }
                    let category_str = match category.as_str() {
                        "dungeons" => "Mythic+",
                        "raids" => "Raiding",
                        _ => category,
                    };
                    msg_send.push_str(&format!("**__{}__**\n", category_str));
                    if let Some(options) = options.as_object() {
                        for (slot, value) in options {
                            let slot_number = match slot.as_str() {
                                "option_1" => "1",
                                "option_2" => "2",
                                "option_3" => "3",
                                _ => slot,
                            };
                            let value_str = match value {
                                Value::Null => "Empty".to_string(),
                                _ => value.to_string(),
                            };
                            msg_send.push_str(&format!("\t\t\t**{}: {}**\n", slot_number, value_str));
                        }
                    }
                }
            }
            
            break; // Exit the loop after finding the character
        }

        if !character_found {
            msg_send = format!("Character not found on roster with the name '{}'.", name);
        }
    } else {
        msg_send = "No character data available.".to_string();
    }

    ctx.say(msg_send).await?;

    Ok(())
}



























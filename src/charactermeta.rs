use serde::Deserialize;
use chrono::{DateTime, Utc};
use crate::config;

#[derive(Debug, Deserialize)]
pub struct CharacterMetaData {
    id: u64,
    name: String,
    realm: String,
    #[serde(rename = "class")]
    class_name: String,
    role: String,
    rank: String,
    status: String,
    note: Option<String>,
    blizzard_id: u64,
    tracking_since: DateTime<Utc>,
}

trait SortCharacterMetaData {
   fn group_with_note(&self) -> Vec<(String, String)>; 
}


impl SortCharacterMetaData for Vec<CharacterMetaData> {
    fn group_with_note(&self) -> Vec<(String, String)> {
        self.iter()
            .filter_map(|c| {
                c.note
                    .as_ref()
                    .map(|note| (c.name.clone(), note.clone()))
            })
        .collect()
    }
}

async fn character_metadata(token: &str) -> Result<Vec<CharacterMetaData>, Box<dyn std::error::Error>> {
    let url = "https://wowaudit.com/v1/characters";
    let client = crate::reqwest::Client::new();

    let metadata: Vec<CharacterMetaData> = client
        .get(url)
        .header("Accept", "application/json")
        .header("Authorization", token)
        .send()
        .await?
        .json()
        .await?;
    Ok(metadata) 
}

pub async fn sorted_date() -> Result<(), Box<dyn std::error::Error>> {
    let teams = [
        ("Mains", config::mains()), 
        ("Alts", config::alts()),
        ("RaiderAlts", config::raider_alts()),
        ("OfficerAlter", config::officer_alts()),

    ];
    let mut all_characters = Vec::new();

    for (team_name, team_token) in teams {
        match character_metadata(&team_token).await {
            Ok(response) => {
                all_characters.extend(response.group_with_note());
            }
            Err(e) => eprintln!("Error fetching {}: {}", team_name, e),
        }
    }
    for (name, note) in all_characters {
        println!("{}: {}", note, name);
    }
    Ok(())
}

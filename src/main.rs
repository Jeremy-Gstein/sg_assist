use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

mod config;
mod parser;

type Error = Box<dyn std::error::Error>;
type AliasMap = HashMap<String, Vec<String>>;


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
    fn group_with_all_metadata(&self) -> Vec<(String, String)>; 

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
    fn group_with_all_metadata(&self) -> Vec<(String, String)> {
        self.iter()
            .map(|c| {
                let metadata_feilds = [
                    Some(format!("ID: {}", c.id.to_string())),
                    Some(format!("Blizzard ID: {}", c.blizzard_id.to_string())),
                    Some(format!("Realm: {}", c.realm)),
                    Some(format!("Class: {}", c.class_name)),
                    Some(format!("Role: {}", c.role)),
                    Some(format!("Rank: {}", c.rank)),
                    Some(format!("Status: {}", c.status)),
                    Some(format!("Tracking Since: {}", c.tracking_since)),
                    c.note.as_ref().map(|note| format!("Note (Alias/nickname): {}", note)),
                ];

                let summary = metadata_feilds
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(", ");
                (c.name.clone(), summary)
            })
        .collect()
    }

}

async fn character_metadata(token: &str) -> Result<Vec<CharacterMetaData>, Error> {
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

pub async fn map_character_metadata() -> Result<Vec<(String, String)>, Error> {
    let teams = [
        ("Mains", config::mains()), 
        ("Alts", config::alts()),
        ("RaiderAlts", config::raider_alts()),
        ("OfficerAlter", config::officer_alts()),

    ];
   
    let mut all_characters: Vec<(String, String)> = Vec::new();
    for (team_name, team_token) in teams {
        match character_metadata(&team_token).await {
            Ok(response) => {
                all_characters.extend(response.group_with_all_metadata());
            }
            Err(e) => eprintln!("Error fetching {}: {}", team_name, e),
        }
    }
    Ok(all_characters)
}
pub async fn map_character_alias() -> Result<AliasMap, Error> {
    let teams = [
        ("Mains", config::mains()), 
        ("Alts", config::alts()),
        ("RaiderAlts", config::raider_alts()),
        ("OfficerAlter", config::officer_alts()),

    ];

    let mut all_characters = Vec::new();
    let mut alias: HashMap<String, Vec<String>> = HashMap::new(); 

    for (team_name, team_token) in teams {
        match character_metadata(&team_token).await {
            Ok(response) => {
                all_characters.extend(response.group_with_note());
            }
            Err(e) => eprintln!("Error fetching {}: {}", team_name, e),
        }
    }
    for (name, note) in all_characters {
        // make sure we map characters with a note 
        if !note.trim().is_empty() {
            alias.entry(note).or_default().push(name);
        }
    }
    Ok(alias)
}



#[tokio::main]
async fn main() -> Result<(), Error> {
    let map_alias = map_character_alias().await;
    dbg!(&map_alias);
    let _ = parser::write_yaml(&map_alias.unwrap());
    //let map_metadata = map_character_metadata().await;
    //dbg!(&map_metadata);

    Ok(())
}

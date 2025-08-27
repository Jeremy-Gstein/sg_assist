use std::sync::OnceLock;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;

// hashmap of all k,v pairs in file 
static TOKENS: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Takes a file expecting 'key=value' and returns a hashmap of k,v pairs in file.
fn parse_token_file(path: &str) -> std::io::Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut map = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        if let Some((name, token)) = line.split_once('=') {
            map.insert(name.trim().to_string(), token.trim().to_string());
        }
    }
    Ok(map)
}

// parse .env file into a hashmap
// this is stored in sg_assist/store_leaderboard so .env exists ../.env
fn load_tokens() -> &'static HashMap<String, String> {
    TOKENS.get_or_init(|| {
        parse_token_file("../.env").unwrap_or_else(|_| {
            eprintln!("WARN: Failed to parse .env file. Falling back to empty map");
            HashMap::new()
        })
    })
}

// trys to get a token by:
//   1. checking if env var exists
//   2. checking if entry exists in .env
fn get_token(name: &str) -> Option<String> {
    env::var(name).ok().or_else(|| load_tokens().get(name).cloned())
}

/// Returns a token value for a specific team.
pub fn mains() -> String{
    get_token("MAINS").expect("Missing TEAM_1 Token")
}

pub fn alts() -> String {
    get_token("ALTS").expect("Missing TEAM_2 Token")

}
pub fn raider_alts() -> String {
    get_token("RAIDER_ALTS").expect("Missing TEAM_3 Token")
}
pub fn officer_alts() -> String {
    get_token("OFFICER_ALTS").expect("Missing TEAM_3 Token")
}


// load alt config from yaml.. map 'Main<-Alts'
// this is stored in sg_assist/store_leaderboard so sg-alts.yaml exists ../../
pub fn load_alt_config() -> HashMap<String, Vec<String>> {
    let config_str = include_str!("../../alts.yaml");
    serde_yaml::from_str(config_str).expect("failed to parse alt config")
}

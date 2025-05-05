use std::sync::OnceLock;
use std::env;
use dotenv::dotenv;
use std::collections::HashMap;

pub fn check_dotenv() {
    dotenv().ok().expect("Missing .env file");
}


pub fn team_1() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        env::var("WOWAUDIT_TOKEN").expect("Missing WOWAUDIT_TOKEN")
    })
}

pub fn team_2() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        env::var("WOWAUDIT_TOKEN_1").expect("Missing WOWAUDIT_TOKEN")
    })
}


// load alt config from yaml.. map 'Main<-Alts'
pub fn load_alt_config() -> HashMap<String, Vec<String>> {
    let config_str = include_str!("../sg-alts.yaml");
    serde_yaml::from_str(config_str).expect("failed to parse alt config")
}


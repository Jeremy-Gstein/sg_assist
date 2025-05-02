use std::sync::OnceLock;
use std::env;


pub fn discord_token() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        env::var("DISCORD_TOKEN").expect("Missing WOWAUDIT_TOKEN")
    })
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



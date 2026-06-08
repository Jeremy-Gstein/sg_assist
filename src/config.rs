use std::sync::OnceLock;
use std::env;

pub fn discord_token() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        env::var("DISCORD_TOKEN")
            .expect("Missing DISCORD_TOKEN")
    })
}

pub fn wowaudit_token() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        env::var("WOWAUDIT_TOKEN")
            .expect("Missing WOWAUDIT_TOKEN")
    })
}

// API endpoint for mplus tracker
pub fn tracker_url() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        env::var("MPLUS_TRACKER_URL")
            .expect("Missing MPLUS_TRACKER_URL")
    })
}

// API only serves requests with auth header using a token generated on api server.
pub fn api_token() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| {
        env::var("MPLUS_API_TOKEN")
            .expect("Missing MPLUS_API_TOKEN")
    })
}

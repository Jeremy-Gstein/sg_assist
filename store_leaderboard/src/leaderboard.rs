use colored::Colorize;
use std::collections::HashMap;
use reqwest;
use serde::{Deserialize, Serialize};
use crate::config;


// #[derive(Debug)]
// pub struct DungeonRun {
//     level: i32,
//     name: String,
// }
//

#[derive(Debug, Deserialize, Serialize)]
pub struct DungeonDone {
    level: i32,
    dungeon: i32,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterData {
    dungeons_done: Option<Vec<DungeonDone>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Character {
    id: i32,
    name: String,
    realm: String,
    data: Option<CharacterData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    period: i32,
    characters: Vec<Character>,
}



async fn fetch_endpoint(token: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let url = format!("https://www.wowaudit.com/v1/historical_data");
    let client = reqwest::Client::new();
 
    let res = client
        .get(&url)
        .header("Accept", "application/json")
        .header("Authorization", token)
        .send()
        .await?
        .error_for_status()?;
    let body = res.text().await?;
    serde_json::from_str(&body).map_err(Into::into)
}


pub async fn get_data() -> Result<Vec<(String, usize)>, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let teams = [
        ("Mains", config::mains()), 
        ("Alts", config::alts()),
        ("RaiderAlts", config::raider_alts()),
        ("OfficerAlts", config::officer_alts()),

    ];
    let mut all_characters = Vec::new();

    let alt_config = config::load_alt_config();
    let alt_to_main: HashMap<_, _> = alt_config
        .iter()
        .flat_map(|(main,alts)| alts.iter().map(move |alt| (alt.as_str(), main.as_str())))
        .collect();

    //fetch data from both teams
    for (team_name, team) in teams {
        let team_start = std::time::Instant::now();
        match fetch_endpoint(&team).await {
            Ok(response) => {
                let duration = team_start.elapsed();
                println!("[{}ms] Fetched data for {}", duration.as_millis(), &team_name);
                all_characters.extend(response.characters);
            }
            Err(e) => eprintln!("Error fetching team: {}", e),
        }
    }

    // init tracking structures
    let mut main_scores: HashMap<String, usize> = HashMap::new();
    let mut total_runs = 0;

    // process characters
    for character in &all_characters {
        let name = character.name.as_str();
        if let Some(data) = &character.data {
            if let Some(dungeons_done) = &data.dungeons_done {
                let mut valid_runs = 0;
                for dungeon in dungeons_done {
                    if dungeon.level >= 1 {
                        valid_runs += 1;
                    }
                }
                if valid_runs > 0 {
                    let main = alt_to_main.get(name)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| name.to_string());

                    // Update scores
                    *main_scores.entry(main.to_string()).or_insert(0) += valid_runs;
                    total_runs += valid_runs;

                }
            }
        }
    }
    let total_duration = start_time.elapsed();
    println!("[{}ms] Processed {} characters", 
        total_duration.as_millis().to_string().bright_yellow(),
        all_characters.len().to_string().bright_magenta(),
    );

    // Convert to sorted leaderboard
    let mut leaderboard: Vec<(String, usize)> = main_scores.into_iter().collect();
    leaderboard.sort_by(|a, b| b.1.cmp(&a.1));

    // Print leaderboard cursed formatting but looks okay (:
    let sep = format!("{}{}{}", "+".bright_yellow(), "--------------------".green(), "+".bright_yellow());
    let rule = format!("{}", "|--------------------|".green());
    println!("{}", sep);
    println!("{}", format!("| {} |", "Mythic+Leaderboard".yellow()).green());
    println!("{}", format!("| {:<12} {:<4}  |", "Character".bright_blue(), "Keys".bright_magenta()).green());
    println!("{}", rule);

    for (main, score) in &leaderboard {
        println!("{}", format!("| - {:<12} {:<3} |", main.bright_blue(), score.to_string().bright_magenta()).green());
    }
    println!("{}", rule);
    println!("{}", format!("|   {} {:<4}      |", "Total:".bright_magenta(), total_runs.to_string().bright_yellow()).green());
    println!("{}", sep);
    Ok(leaderboard)

}

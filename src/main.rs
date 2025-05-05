use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use colored::Colorize;

mod config;



#[derive(Debug)]
struct DungeonRun {
    level: i32,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DungeonDone {
    level: i32,
    dungeon: i32,
}


#[derive(Debug, Deserialize, Serialize)]
struct CharacterData {
    dungeons_done: Option<Vec<DungeonDone>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Character {
    id: i32,
    name: String,
    realm: String,
    data: Option<CharacterData>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    period: i32,
    characters: Vec<Character>,
}



fn get_dungeon_mapping() -> HashMap<i32, String> {
    let mut mapping = HashMap::new();
    // season 1 
    mapping.insert(375, "Mists of Tirna Scithe".to_string());
    mapping.insert(502, "City of Threads".to_string());
    mapping.insert(503, "Ara-Kara, City of Echoes".to_string());
    mapping.insert(505, "Dawnbreaker".to_string());
    mapping.insert(507, "Grim Batol".to_string());
    mapping.insert(353, "Seige of Boralus".to_string());
    mapping.insert(501, "Stonevault".to_string());
    mapping.insert(376, "Nercrotic Wake".to_string());
    // season 2
    mapping.insert(506, "Cinderbrew Meadery".to_string());
    mapping.insert(504, "Darkflame Cleft".to_string());
    mapping.insert(370, "Mechagon Workshop".to_string());
    mapping.insert(525, "Floodgate".to_string());
    mapping.insert(499, "Priory of Sacred Flame".to_string());
    mapping.insert(247, "Motherload".to_string());
    mapping.insert(500, "Rookery".to_string());
    mapping.insert(382, "Theater of Pain".to_string());
    mapping
}


fn rename_dungeon(id: i32, mapping: &HashMap<i32,String>) -> Option<String> {
    mapping.get(&id).cloned()
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


async fn call_wowaudit_api(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();


    let teams = [config::team_1(), config::team_2()];
    let mut all_characters = Vec::new();
    let dungeon_mapping = get_dungeon_mapping();

    let alt_config = config::load_alt_config();
    let alt_to_main: HashMap<_, _> = alt_config
        .iter()
        .flat_map(|(main, alts)| alts.iter().map(move |alt| (alt.as_str(), main.as_str())))
        .collect();

    // Fetch data from both teams
    for team in teams {
        let team_start = std::time::Instant::now();
        match fetch_endpoint(team).await {
            Ok(response) => {
                let duration = team_start.elapsed();
                println!("[{}ms] Fetched data for team", duration.as_millis());
                all_characters.extend(response.characters);
            } 
            Err(e) => eprintln!("Error fetching team: {}", e),
        }
    }

    // Initialize tracking structures
    let mut main_scores: HashMap<String, usize> = HashMap::new(); // Use owned Strings
    let mut total_runs = 0;

    // Process characters
    for character in &all_characters {
        let name = character.name.as_str();
        if let Some(data) = &character.data {
            if let Some(dungeons_done) = &data.dungeons_done {
                let mut runs = Vec::new();
                let mut valid_runs = 0;

                for dungeon in dungeons_done {
                    if dungeon.level >= 1 {
                        valid_runs += 1;
                        let dungeon_name = rename_dungeon(dungeon.dungeon, &dungeon_mapping)
                            .unwrap_or_else(|| format!("Unknown Dungeon ID: ({})", dungeon.dungeon));
                    
                        runs.push(DungeonRun {
                            level: dungeon.level,
                            name:  dungeon_name,
                        });
                    }
                }
               
                if valid_runs > 0 {
                    // Get main name or use character name
                    let main = alt_to_main.get(name)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| name.to_string());

                    // Update scores
                    *main_scores.entry(main.to_string()).or_insert(0) += valid_runs;
                    total_runs += valid_runs;
                    // Verbose logging
                    if verbose {
                        let hr = "+--------------------------------------------+";
                        println!("\n{}", hr);
                        println!(
                            "| Character: {:<12} Main: {:<12} |",
                            name.bright_blue(), main.blue().bold()
                        );
                        println!(
                            "| Total runs: {:<3}                            |",
                            valid_runs.to_string().yellow()
                        );
                        println!("|--------------------------------------------|");
                        for run in runs {
                            println!("| {:<25} +{:<3}             |", 
                                run.name.bright_green(), run.level.to_string().bright_magenta()
                            );
                        }
                        println!("{}", hr);
                    } else {
                        // Print character details
                        println!("+---------------------------------------+");
                        println!("|    Character: {:<12.12} Total: {:>2}  |", name.bright_blue(), valid_runs.to_string().yellow());
                        println!("+---------------------------------------+");
                    }
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

    for (main, score) in leaderboard {
        println!("{}", format!("| - {:<12} {:<3} |", main.bright_blue(), score.to_string().bright_magenta()).green());
    }
    println!("{}", rule);
    println!("{}", format!("|   {} {:<4}      |", "Total:".bright_magenta(), total_runs.to_string().bright_yellow()).green());
    println!("{}", sep);
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SG Assistant Debug for /keysdone");
    let args: Vec<String> = std::env::args().collect();
    // load .env
    config::check_dotenv();
    // Default to verbose on Windows if no args
    let (is_verbose, is_windows_default) = {
        #[cfg(windows)] {
            (args.len() == 1 || matches!(args.get(1).map(|s| s.as_str()), Some("--verbose") | Some("-v")), args.len() == 1)
        }
        #[cfg(not(windows))] {
            (matches!(args.get(1).map(|s| s.as_str()), Some("--verbose") | Some("-v")), false)
        }
    };
    // Run the program
    let result = if is_verbose {
        if is_windows_default {
            println!("Running with verbose data (Windows default)");
        } else {
            println!("Running with verbose data");
        }
        call_wowaudit_api(true).await
    } else {
        println!("Running in standard mode");
        call_wowaudit_api(false).await
    };

    // Windows pause
    #[cfg(windows)] {
        use std::io::{self, Write};
        let mut input = String::new();
        print!("Press Enter to exit...");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
    }

    result
}
 

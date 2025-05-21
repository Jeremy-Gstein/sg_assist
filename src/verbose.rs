use colored::Colorize;
use std::collections::HashMap;
use crate::DungeonRun;
use crate::rename_dungeon;
use crate::get_dungeon_mapping;
use crate::fetch_endpoint;
use crate::config;

/// Expects a bool, when true, we print 'if verbose'
/// verbose includes character name, total, and list of exact dungeons completed that week.
pub async fn call_wowaudit_api(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    let teams = [
        ("Mains", config::mains()), 
        ("Alts", config::alts()),
        ("RaiderAlts", config::raider_alts()),
        ("OfficerAlter", config::officer_alts()),

    ];
 
    let mut all_characters = Vec::new();
    let dungeon_mapping = get_dungeon_mapping();

    let alt_config = config::load_alt_config();
    let alt_to_main: HashMap<_, _> = alt_config
        .iter()
        .flat_map(|(main, alts)| alts.iter().map(move |alt| (alt.as_str(), main.as_str())))
        .collect();

    // Fetch data from both teams
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

    // Initialize tracking structures
    let mut main_scores: HashMap<String, usize> = HashMap::new(); // Use owned Strings
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
   Ok(())
}

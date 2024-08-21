use crate::Context; // Use the Context type alias from main.rs
use crate::Error; // Use the Error type alias from main.rs
use serde::{Deserialize, Serialize};
use std::fmt;


#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    #[serde(rename = "Character Name")]
    name: String,
    #[serde(rename = "Number of Entries")]
    num: u64,
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "**{} : {}**", self.name, self.num)
    }
}


fn format_with_commas(number: u64) -> String {
    let num_str = number.to_string();
    let mut result = String::new();
    let mut chars = num_str.chars().rev().peekable();

    while let Some(c) = chars.next() {
        result.push(c);
        if chars.peek().is_some() && result.len() % 4 == 3 {
            result.push(',');
        }
    }

    result.chars().rev().collect()
}

/// get the current entries from google sheet. 
#[poise::command(slash_command, broadcast_typing)]
pub async fn jackpot(ctx: Context<'_>) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let res = client.get("https://docs.google.com/spreadsheets/d/1fVPBtzSeZrGlUYHrirVTeYqNZWE4EHlDhRNOf4SADu4/gviz/tq?tqx=out:csv").send().await?;
    let csv_text = res.text().await?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_text.as_bytes());

    let mut records = reader
        .deserialize::<Record>()
        .collect::<Result<Vec<Record>, csv::Error>>()?;
    records.sort_by(|a, b| b.num.cmp(&a.num));
    
    let total_sum: u64 = records.iter().map(|record| record.num).sum();
    let total_gold = total_sum * 1000u64; 
    let format_gold = format_with_commas(total_gold);
    let mut msg_send = String::new();
   
    msg_send.push_str(&format!("## MegaGewdOver9000 Jackpot!\n"));
    msg_send.push_str(&format!("## **Gold in pool: __{}__!**\n", format_gold));
    msg_send.push_str(&format!("**__Name : Entries__**\n"));
    for record in records {
        msg_send.push_str(&format!("- {}\n", record));
    }

    ctx.say(msg_send).await?;
    Ok(())

}

use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

type AliasMap = HashMap<String, Vec<String>>;
type Error = Box<dyn std::error::Error>;

/// takes a hashmap of k=(alias/nickname), v=(array of characters)
/// writes the hashmap to yaml format.
/// # Example:
/// ```yaml
/// Alias:
///   - foo
///   - bar
/// ```
pub fn write_yaml(alias_map: &AliasMap) -> Result<(), Error> {
    let mut file = File::create("alias.yaml")?;

    for (key, values) in alias_map {
        if key.trim().is_empty() {
            continue;
        }
        writeln!(file, "{}:", escape_yaml_key(key))?;
        for value in values {
            writeln!(file, "  - {}", escape_yaml_value(value))?;
        }
    }

    Ok(())
}

fn escape_yaml_key(s: &str) -> String {
    if s.contains(':') || s.contains('"') || s.contains('#') || s.trim().is_empty() {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }

}

fn escape_yaml_value(s: &str) -> String {
    if s.contains(':') || s.contains('"') || s.contains('#') || s.trim().is_empty() {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

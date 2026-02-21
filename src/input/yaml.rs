use anyhow::Result;
use std::path::Path;

use super::models::InputFile;

pub fn parse(path: &Path) -> Result<InputFile> {
    let content = std::fs::read_to_string(path)?;
    let input: InputFile = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML file '{}': {}", path.display(), e))?;
    Ok(input)
}

use anyhow::{bail, Result};
use std::path::PathBuf;

pub struct Config {
    pub api_token: String,
}

/// Config file schema (`~/.config/bypass/config.yaml`).
#[derive(serde::Deserialize)]
struct ConfigFile {
    api_token: Option<String>,
}

impl Config {
    pub fn load(cli_token: Option<String>) -> Result<Self> {
        // Priority: CLI flag > env var (handled by clap) > config file.
        if let Some(t) = cli_token {
            return Ok(Config { api_token: t });
        }

        // Fall through to config file.
        if let Some(path) = config_file_path() {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(cfg) = serde_yaml::from_str::<ConfigFile>(&content) {
                    if let Some(t) = cfg.api_token {
                        if !t.is_empty() {
                            return Ok(Config { api_token: t });
                        }
                    }
                }
            }
        }

        bail!(
            "No API token found.\n\
             Provide it via:\n  \
             • --token <TOKEN>\n  \
             • SHORTCUT_API_TOKEN environment variable\n  \
             • ~/.config/bypass/config.yaml  (field: api_token)"
        )
    }
}

fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("bypass").join("config.yaml"))
}

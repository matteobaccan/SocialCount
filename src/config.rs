// SocialCount - Real-time YouTube Channel Statistics
// by Matteo Baccan
// https://github.com/matteobaccan/SocialCount

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_FILENAME: &str = "channels.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_channels")]
    pub channels: Vec<String>,

    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,

    #[serde(default = "default_card_opacity")]
    pub card_opacity: u8,
}

fn default_channels() -> Vec<String> {
    vec!["@matteobaccan".to_string()]
}

fn default_refresh_interval() -> u64 {
    300
}

fn default_card_opacity() -> u8 {
    190
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channels: default_channels(),
            refresh_interval_secs: default_refresh_interval(),
            card_opacity: default_card_opacity(),
        }
    }
}

impl Config {
    pub fn load_or_create() -> Self {
        let path = Path::new(CONFIG_FILENAME);
        if !path.exists() {
            let default_cfg = Config::default();
            if let Ok(content) = toml::to_string_pretty(&default_cfg) {
                let _ = fs::write(path, content);
            }
            return default_cfg;
        }

        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<Config>(&content) {
                Ok(mut cfg) => {
                    // Normalize handles
                    cfg.channels = cfg
                        .channels
                        .into_iter()
                        .map(|ch| {
                            let trimmed = ch.trim();
                            if !trimmed.starts_with('@') && !trimmed.starts_with("UC") {
                                format!("@{}", trimmed)
                            } else {
                                trimmed.to_string()
                            }
                        })
                        .filter(|ch| !ch.is_empty())
                        .collect();
                    cfg
                }
                Err(err) => {
                    eprintln!("Errore parsing {}: {}. Uso valori default.", CONFIG_FILENAME, err);
                    Config::default()
                }
            },
            Err(err) => {
                eprintln!("Errore lettura {}: {}. Uso valori default.", CONFIG_FILENAME, err);
                Config::default()
            }
        }
    }
}

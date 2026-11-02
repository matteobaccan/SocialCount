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

    #[serde(default = "default_compact_mode")]
    pub compact_mode: bool,

    #[serde(default = "default_window_width")]
    pub window_width: f32,

    #[serde(default = "default_window_height")]
    pub window_height: f32,
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

fn default_compact_mode() -> bool {
    false
}

fn default_window_width() -> f32 {
    360.0
}

fn default_window_height() -> f32 {
    440.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channels: default_channels(),
            refresh_interval_secs: default_refresh_interval(),
            card_opacity: default_card_opacity(),
            compact_mode: default_compact_mode(),
            window_width: default_window_width(),
            window_height: default_window_height(),
        }
    }
}

impl Config {
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Path::new(CONFIG_FILENAME);
        let channels_formatted = self
            .channels
            .iter()
            .map(|c| format!("    \"{}\"", c))
            .collect::<Vec<_>>()
            .join(",\n");

        let content = format!(
            r#"# SocialCount configuration file
# List of YouTube channels to monitor (handles with or without @)
channels = [
{channels_formatted}
]

# Automatic refresh interval in seconds (default: 300 = 5 minutes)
refresh_interval_secs = {refresh}

# Background opacity (0 = fully transparent, 255 = opaque, default: 190)
card_opacity = {opacity}

# Compact mode (true = compact widget, false = full view)
compact_mode = {compact}

# Window dimensions (width and height in logical pixels)
window_width = {width:.1}
window_height = {height:.1}
"#,
            channels_formatted = channels_formatted,
            refresh = self.refresh_interval_secs,
            opacity = self.card_opacity,
            compact = self.compact_mode,
            width = self.window_width.max(200.0),
            height = self.window_height.max(50.0),
        );
        fs::write(path, content)
    }

    pub fn load_or_create() -> Self {
        let path = Path::new(CONFIG_FILENAME);
        if !path.exists() {
            let default_cfg = Config::default();
            let _ = default_cfg.save();
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
                    if cfg.window_width < 200.0 {
                        cfg.window_width = 360.0;
                    }
                    if cfg.window_height < 50.0 {
                        cfg.window_height = 440.0;
                    }
                    cfg
                }
                Err(err) => {
                    eprintln!(
                        "Error parsing {}: {}. Using default values.",
                        CONFIG_FILENAME, err
                    );
                    Config::default()
                }
            },
            Err(err) => {
                eprintln!(
                    "Error reading {}: {}. Using default values.",
                    CONFIG_FILENAME, err
                );
                Config::default()
            }
        }
    }
}

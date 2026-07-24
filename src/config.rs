use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hide_offset: i32,
    pub debug: bool,
    pub ignore: Vec<String>,
    #[serde(default = "default_ignore_hover")]
    pub ignore_hover: bool,
}

fn default_ignore_hover() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hide_offset: 300,
            debug: false,
            ignore: vec![],
            ignore_hover: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let paths = [
            dirs_config_path(),
            Some(PathBuf::from("config.toml")),
        ];
        for path in paths.into_iter().flatten() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    log::info!("Loaded config from {}", path.display());
                    return config;
                }
            }
        }
        log::info!("Using default config");
        Config::default()
    }

    pub fn should_ignore(&self, class: &str, title: &str) -> bool {
        let class_lower = class.to_lowercase();
        let title_lower = title.to_lowercase();
        self.ignore.iter().any(|s| {
            let s_lower = s.to_lowercase();
            class_lower.contains(&s_lower) || title_lower.contains(&s_lower)
        })
    }
}

fn dirs_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/shy/config.toml"))
}

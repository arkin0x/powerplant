// config.rs

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub num_threads: usize,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

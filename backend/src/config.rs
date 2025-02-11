use serde_derive::Deserialize;
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub app: App,
    pub database: Database,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub address: String,
    pub username: String,
    pub user_password: String,
    pub db_name: String,
}

#[derive(Debug, Deserialize)]
pub struct App {
    pub address: String,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read the configuration file: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to parse the configuration file as TOML: {0}")]
    TomlParse(#[from] toml::de::Error),
}

pub fn load_config(file_path: &str) -> Result<Config, ConfigError> {
    let config_content = fs::read_to_string(file_path)?;

    let config: Config = toml::de::from_str(&config_content)?;

    Ok(config)
}

use std::path::PathBuf;
use config::Config;

pub mod config;
pub mod token;
pub mod lexer;
pub mod node;
pub mod parser;
pub mod validate;
pub mod compile;
pub mod send;
pub mod definitions;
pub mod utility;

pub struct ConfigFileNotFoundError {}

pub fn load_config(file: &PathBuf) -> Result<Config, ConfigFileNotFoundError> {
    let data = if !file.exists() {
        return Err(ConfigFileNotFoundError {})
    } else {
        std::fs::read_to_string(file).expect("No config file")
    };

    match toml::from_str(&data) {
        Ok(res) => Ok(res),
        Err(err) => panic!("Failed to parse config: {}", err)
    }
}
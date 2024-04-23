use std::path::Path;
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

pub fn load_config() -> Config {
    let data;
    if !Path::new("test_project/dfrs.toml").exists() {
        data = String::from("");
    } else {
        data = std::fs::read_to_string("test_project/dfrs.toml").expect("No config file");
    }

    match toml::from_str(&data) {
        Ok(res) => return res,
        Err(err) => panic!("Failed to parse config: {}", err)
    }
}
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
  #[serde(default)]
  pub sending: Sending,
  #[serde(default)]
  pub debug: Debug,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Sending {
  pub api: SendApi,
}

impl Default for Sending {
  fn default() -> Self {
    Sending {
      api: SendApi::CodeClientGive,
    }
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum SendApi {
  #[serde(rename="codeclient-give")]
  CodeClientGive,
  #[serde(rename="codeclient-place")]
  CodeClientPlace,
  #[serde(rename="recode")]
  Recode,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Debug {
  #[serde(default = "bool::default")]
  pub tokens: bool,
  #[serde(default = "bool::default")]
  pub nodes: bool,
  #[serde(default = "bool::default")]
  pub compile: bool,
  #[serde(default = "bool::default")]
  pub connection: bool,
}

impl Config {
  pub fn save(&self, path: &PathBuf) {
    let data = toml::to_string(self).expect("Failed to create new config");
    std::fs::write(path, data).expect("Failed to save new config");
  }
}

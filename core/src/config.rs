use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub sending: Sending,
    #[serde(default)]
    pub debug: Debug
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Sending {
    pub api: SendApi
}

impl Default for Sending {
    fn default() -> Self {
        Sending { api: SendApi::CodeClient }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all="lowercase")]
pub enum SendApi {
    CodeClient,
    Recode
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Debug {
    #[serde(default = "bool::default")]
    pub tokens: bool,
    #[serde(default = "bool::default")]
    pub nodes: bool,
    #[serde(default = "bool::default")]
    pub compile: bool,
    #[serde(default = "bool::default")]
    pub connection: bool
}

impl Default for Debug {
    fn default() -> Self {
        Self { tokens: false, nodes: false, compile: false, connection: false }
    }
}
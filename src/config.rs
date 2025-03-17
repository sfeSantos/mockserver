use serde::Deserialize;
use std::{collections::HashMap, fs};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct Endpoint {
    pub method: Vec<String>,
    pub file: String,
    pub status_code: Option<u16>,
    pub authentication: Option<Value>,
}

pub type Config = HashMap<String, Endpoint>;

pub fn load_config(config_file: &str) -> anyhow::Result<Config> {
    let config_data = fs::read_to_string(config_file)?;
    let config: Config = serde_yaml::from_str(&config_data)?;

    Ok(config)
}
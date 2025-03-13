use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Debug, Deserialize, Clone)]
pub struct Endpoint {
    pub method: Vec<String>,
    pub file: String
}

pub type Config = HashMap<String, Endpoint>;

pub fn load_config() -> anyhow::Result<Config> {
    let config_data = fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&config_data)?;

    Ok(config)
}
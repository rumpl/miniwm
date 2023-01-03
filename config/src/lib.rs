use std::{
    fs::File,
    io::{self, BufReader},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error("config not found")]
    ConfigNotFound(#[from] io::Error),

    #[error("invalid config")]
    InvalidConfig(#[from] serde_yaml::Error),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub startup: Vec<String>,
}

pub fn load_config() -> Result<Config, ConfigLoadError> {
    let file = File::open("config.yaml")?;
    let reader = BufReader::new(file);

    let config: Config = serde_yaml::from_reader(reader)?;
    Ok(config)
}

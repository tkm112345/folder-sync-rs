use serde::Deserialize;
use std::{io::Read,fs};

#[derive(Deserialize)]
pub struct AppConfig {
    pub bts : Vec<BtsConfig>,
    pub cdf : CdfConfig,
}

#[derive(Deserialize,Clone)]
pub struct BtsConfig {
    pub source : String,
    pub destination : String,
    pub overwrite: bool,
    pub exclude : Vec<String>,
}

#[derive(Deserialize)]
pub struct CdfConfig {
    pub source : String,
    pub destination : String,
}


pub fn load_config(path : &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config : AppConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

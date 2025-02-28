use crate::messages::*;
use serde::Deserialize;
use std::{io::Read,fs};

#[derive(Deserialize)]
pub struct AppConfig {
    pub bts : BtsConfigWrapper,
    pub cdf : CdfConfig,
}

#[derive(Deserialize)]
pub struct BtsConfigWrapper {
    pub configs : Vec<BtsConfig>,
    pub exclude : Vec<String>,
}

#[derive(Deserialize,Clone)]
pub struct BtsConfig {
    pub source : String,
    pub destination : String,
    pub overwrite: bool
}

#[derive(Deserialize)]
pub struct CdfConfig {
    pub source : String,
    pub destination : String,
}


pub fn load_config(file_name : &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    // 実行ファイルのディレクトリを取得
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().expect(ERR_FAILED_TO_GET_DIRECTORY);


    // config.jsonのパスを生成
    let config_path = exe_dir.join(file_name);
    let mut file = fs::File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config : AppConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

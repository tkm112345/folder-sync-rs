// cspell:ignore simplelog PKGNAME indifatif
mod config;
mod commands;
mod backup;
mod folders;
mod utils;

use clap::Parser;
use log::info;
use std::io::{self, Write};
use log4rs;
use std::sync::Arc;

use crate::config::load_config;
use crate::commands::{Cli,Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // ログフォルダが存在しない場合は作成
    if !std::path::Path::new("./log").exists() {
        std::fs::create_dir_all("./log")?;
    }

    // ログの設定
    log4rs::init_file("log4rs.yaml",Default::default())?;
    
    info!("Start folder sync app");

    let cli = Cli::parse();
    let config = load_config("./config.json").expect("Failed to load config.json");
    let config = Arc::new(config);

    match &cli.command {
        Commands::BackupToSsd => {
            info!("Backup mode");
            backup::execute_backup(&config.bts)?;
        }
        Commands::CreateFolders => {
            info!("Create folders mode");
            folders::execute_create_folders(&config.cdf)?;
        }
    }
    
    info!("Finish folder sync app");

    // ユーザに対して、プログラムの最後にEnterキーの入力を待つ
    println!("Press Enter to exit...");
    let mut input = String::new();
    io::stdout().flush()?; // 標準出力をフラッシュして、メッセージが確実に表示されるようにする
    io::stdin().read_line(&mut input)?;

    Ok(())
}

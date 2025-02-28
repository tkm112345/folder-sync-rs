// cspell:ignore simplelog PKGNAME indifatif
mod config;
mod commands;
mod backup;
mod folders;
mod utils;
mod messages;

use clap::Parser;
use log::info;
use std::io::{self, Write};
use log4rs;
use std::sync::Arc;

use crate::config::load_config;
use crate::commands::{Cli,Commands};
use crate::messages::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // 実行ファイルのディレクトリを取得
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().expect(ERR_FAILED_TO_GET_DIRECTORY);

    // ログフォルダが存在しない場合は作成
    if !std::path::Path::new("./log").exists() {
        std::fs::create_dir_all("./log")?;
    }
    // ログの設定
    let log4rs_config_path = exe_dir.join("log4rs.yaml");
    log4rs::init_file(log4rs_config_path,Default::default())?;
    info!("{}", LOG_START);

    let cli = Cli::parse();
    let config = load_config("config.json").expect(ERR_FAILED_TO_LOAD_CONFIG);
    let config = Arc::new(config);
    match &cli.command {
        Commands::BackupToSsd => {
            info!("{}", LOG_BACKUP_MODE);
            backup::execute_backup(&config.bts)?;
        }
        Commands::CreateFolders => {
            info!("{}", LOG_CREATE_FOLDERS_MODE);
            folders::execute_create_folders(&config.cdf)?;
        }
    }
    
    info!("{}", LOG_FINISH);

    // ユーザに対して、プログラムの最後にEnterキーの入力を待つ
    println!("{}", MSG_PRESS_ENTER_TO_EXIT);
    let mut input = String::new();
    io::stdout().flush()?; // 標準出力をフラッシュして、メッセージが確実に表示されるようにする
    io::stdin().read_line(&mut input)?;

    Ok(())
}

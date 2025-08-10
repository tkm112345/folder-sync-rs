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
use std::time::Instant;

use crate::config::load_config;
use crate::commands::{Cli,Commands};
use crate::messages::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // 処理時間の計測開始
    let start = Instant::now();

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

    let mut cli = Cli::parse();
    let config = load_config("config.json").expect(ERR_FAILED_TO_LOAD_CONFIG);
    let config = Arc::new(config);

    // コマンドライン引数のparse
    if cli.command.is_none() {
        loop {
            println!("Choose mode");
            println!("1 > Backup mode");
            println!("2 > Create folders mode");
            print!("Enter mode number:  ");
            io::stdout().flush()?;

            let mut input = String::new();
            match io::stdin().read_line(&mut input){
                Ok(_) => {
                    let mode = input.trim().parse::<u32>();
                    match mode {
                        Ok(1) => {
                            cli.command = Some(Commands::BackupToSsd);
                            break;
                        }
                        Ok(2) => {
                            cli.command = Some(Commands::CreateFolders);
                            break;
                        }
                        _ => {
                            println!("Invalid mode number. Please enter 1 or 2");
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        println!("Ctrl + C detected. Exiting.");
                        return Ok(());
                    } else {
                        eprintln!("Error reading input : {}",e);
                        return Err(Box::new(e));
                    }
                }
            }
        }
    }


    // // デバッグログを追加して、読み込んだパスを確認
    // for bts_config in &config.bts.configs {
    //     info!("Loaded source path: {}", bts_config.source);
    //     info!("Loaded destination path: {}", bts_config.destination);
    // }


    match &cli.command {
        Some(Commands::BackupToSsd) => {
            info!("{}", LOG_BACKUP_MODE);
            backup::execute_backup(&config.bts)?;
        }
        Some(Commands::CreateFolders) => {
            info!("{}", LOG_CREATE_FOLDERS_MODE);
            folders::execute_create_folders(&config.cdf)?;
        }
        None => unreachable!(),
    }
    
    info!("{}", LOG_FINISH);

    // 処理時間計測終了
    let end = Instant::now();
    let duration = end.duration_since(start);
    let duration_secs = duration.as_secs_f64(); // f64 型の秒数に変換
    let msg_duration = format!("{:.1}", duration_secs); // Durationを文字列に変換
    let result = MSG_EXECUTE_TIME.replace("{}", &msg_duration);
    info!("{}", result.as_str()); // 実行時間ログ追記
    println!("{}", result.as_str()); // 標準出力にも表示

    // ユーザに対して、プログラムの最後にEnterキーの入力を待つ
    println!("{}", MSG_PRESS_ENTER_TO_EXIT);
    let mut input = String::new();
    io::stdout().flush()?; // 標準出力をフラッシュして、メッセージが確実に表示されるようにする
    io::stdin().read_line(&mut input)?;

    Ok(())
}

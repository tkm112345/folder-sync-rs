// cspell:ignore simplelog PKGNAME indifatif
use std::io::{self};
use indicatif::{ProgressBar,ProgressStyle};
use clap::{Parser,Subcommand};
use serde::Deserialize;
use std::{io::Read,io::Write,fs,thread};
use std::path::Path;
use log::{error,info};
use log4rs;
use std::sync::{Arc,mpsc};

const VERSION :&str = env!("CARGO_PKG_VERSION");
const PKGNAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Parser)]
#[clap(
    name = PKGNAME,
    about = DESCRIPTION,
    version = VERSION
)]
struct Cli {
    #[clap(subcommand)]
    command : Commands,

    #[clap(short,long,help = "詳細なログを出力")]
    verbose : bool,
}

#[derive(Subcommand)]
enum Commands {
    /// バックアップ
    #[command(alias = "-bts",name = "--backup-to-ssd")]
    BackupToSsd,
    /// フォルダ構成作成
    #[command(alias = "-cdf",name = "--create-destination-folders")]
    CreateFolders,
}

#[derive(Deserialize)]
struct AppConfig {
    bts : Vec<BtsConfig>,
    cdf : CdfConfig,
}

#[derive(Deserialize,Clone)]
struct BtsConfig {
    source : String,
    destination : String,
    overwrite: bool,
    exclude : Vec<String>,
}

#[derive(Deserialize)]
struct CdfConfig {
    source : String,
    destination : String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // ログフォルダが存在しない場合は作成
    if !std::path::Path::new("./log").exists() {
        fs::create_dir_all("./log")?;
    }

    // ログの設定
    log4rs::init_file("log4rs.yaml",Default::default())?;
    
    info!("Start folder sync app");

    let cli = Cli::parse();
    let config = load_config("./config.json").expect("Failed to load config.json");
    let config = Arc::new(config);

    match &cli.command {
        Commands::BackupToSsd => {
            if cli.verbose {
                info!("Backup mode");
            }
            let mut handles = vec![];
            let total_files = count_files(&config.bts)?;
            let progress_bar = Arc::new(ProgressBar::new(total_files));
            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")?
                    .progress_chars("#>-"),
            );
            progress_bar.set_message("Backing up");

            for bts_config in &config.bts {
                let bts_config = Arc::new(bts_config.clone());
                let progress_bar = Arc::clone(&progress_bar);
                let handle = thread::spawn({
                    let bts_config = Arc::clone(&bts_config);
                    move || {
                        if let Err(err) = backup_to_ssd(&bts_config, &progress_bar){
                            error!("Backup failed : {}",err);
                        }
                    }
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            progress_bar.finish_with_message("Backup complete");
        }
        Commands::CreateFolders => {
            if cli.verbose {
                info!("Create folders mode");
            }
            if let Err(err) = create_folders(&config.cdf){
                error!("Create folders failed: {}", err);
            }
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

fn load_config(path : &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config : AppConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

fn count_files(bts_configs: &[BtsConfig]) -> Result<u64, Box<dyn std::error::Error>> {
    let (tx,rx) = mpsc::channel();
    let mut handles = vec![];

    for config in bts_configs {
        let tx = tx.clone();
        let source_path = Path::new(&config.source).to_path_buf();
        let handle = thread::spawn(move || {
            let count = count_files_recursive(&source_path).unwrap_or(0);
            tx.send(count).unwrap();
        });
        handles.push(handle);
    }

    drop(tx); // メインスレッドの送信側をドロップして、終了を通知
    
    let mut total_files = 0;
    for count in rx {
        total_files += count;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(total_files)

}

fn count_files_recursive(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let mut count = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            count += count_files_recursive(&path)?;
        }
    } else {
        count += 1;
    }
    Ok(count)
}

fn backup_to_ssd(config : &BtsConfig,progress_bar : &Arc<ProgressBar>) -> Result<(),Box<dyn std::error::Error>> {
    // println!("Source : {}, Destination : {}",config.bgs.source, config.bgs.destination);
    // println!("Overwrite : {}, Exclude : {:?}",config.bgs.overwrite, config.bgs.exclude);

    let source_path = Path::new(&config.source);
    let destination_path = Path::new(&config.destination);

    // ソースフォルダが存在するか確認
    if !source_path.exists(){
        return Err(format!("Source folder does not exist : {}", config.source).into());
    }

    // ディレクトリを再帰的にコピー
    copy_recursive(source_path,destination_path,config.overwrite, &config.exclude,progress_bar)?;

    Ok(())
}

fn copy_recursive(
    source : &Path,
    destination : &Path,
    overwrite : bool,
    exclude : &[String],
    progress_bar : &Arc<ProgressBar>
) -> Result<(),Box<dyn std::error::Error>> {
    // 除外リストに含まれる場合はスキップ
    if exclude.iter().any(|e| source.ends_with(e)){
        info!("Skipping excludes item : {}",source.display());
        return Ok(());
    }

    if source.is_dir() {
        // ディレクトリを作成
        fs::create_dir_all(destination)?;

        // ディレクトリ内のアイテムを再帰的にコピー
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let destination = destination.join(entry.file_name());
            copy_recursive(&path, &destination, overwrite, exclude,progress_bar)?;
        }
    } else {
        // ファイルをコピー
        if destination.exists() {
            if !overwrite {
                info!("Skipping existing file: {}", destination.display());
                return Ok(());
            }

            // ファイル名、タイムスタンプ、ファイルサイズを比較
            if let Ok(source_metadata) = fs::metadata(source) {
                if let Ok(destination_metadata) = fs::metadata(&destination) {
                    if source_metadata.len() == destination_metadata.len()
                        && source_metadata.modified()? == destination_metadata.modified()?
                        && source.file_name() == destination.file_name()
                    {
                        info!("Skipping unchanged file: {}", destination.display());
                        return Ok(());
                    }
                }
            }
        }

        fs::copy(source,destination)?;
        info!("Copied: {} to {}", source.display(), destination.display());
        progress_bar.inc(1);
    }

    Ok(())


}

fn create_folders(config : &CdfConfig) -> Result<(),Box<dyn std::error::Error>> {
    // println!("Source : {}, Destination : {}",config.cgf.source, config.cgf.destination);
    let source_path = Path::new(&config.source);
    let destination_path = Path::new(&config.destination);

    // ソースフォルダが存在するか確認
    if !source_path.exists(){
        return Err(format!("Source folder does not exist : {}",config.source).into());
    }
    
    // フォルダ構成を再帰的に作成
    create_folders_recursive(source_path, destination_path)?;

    Ok(())
}

fn create_folders_recursive(
    source:&Path,
    destination: &Path
) -> Result<(),Box<dyn std::error::Error>>{
    if source.is_dir() {

        // destinationにフォルダが既に存在する場合はスキップ
        if !destination.exists(){
            // ディレクトリを作成
            fs::create_dir_all(destination)?;
        }

        // ディレクトリ内のアイテムを再帰的に処理
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let destination = destination.join(entry.file_name());
            create_folders_recursive(&path,&destination)?;
        }
    }

    Ok(())
}
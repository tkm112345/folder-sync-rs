use clap::{Parser,Subcommand};
use serde::Deserialize;
use std::io::Read;
use std::fs;
use std::path::Path;
use log::{error,info, LevelFilter};
use simplelog::*;

#[derive(Parser)]
#[clap(
    name = "sync-file-tool",
    about = "共有フォルダとバックアップ用のSSD間の同期ツール",
    version = "0.1.0"
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
    bts : BgsConfig,
    cdf : CgfConfig,
}

#[derive(Deserialize)]
struct BgsConfig {
    source : String,
    destination : String,
    overwrite: bool,
    exclude : Vec<String>,
}

#[derive(Deserialize)]
struct CgfConfig {
    source : String,
    destination : String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // ログフォルダが存在しない場合は作成
    if !std::path::Path::new("./log").exists() {
        fs::create_dir_all("./log")?;
    }

    // ログファイルを開く(追記モード)
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("./log/main.log")?;

    // ログの設定
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            file,
        ),
    ])?;
    
    info!("Start folder sync app");

    let cli = Cli::parse();
    let config = load_config("./config.json").expect("Failed to load config.json");

    match &cli.command {
        Commands::BackupToSsd => {
            if cli.verbose {
                info!("Backup mode");
            }
            if let Err(err) = backup_to_ssd(&config.bts){
                error!("Backup  failed: {}", err);
            }
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
    Ok(())
}

fn load_config(path : &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config : AppConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

fn backup_to_ssd(config : &BgsConfig) -> Result<(),Box<dyn std::error::Error>> {
    // println!("Source : {}, Destination : {}",config.bgs.source, config.bgs.destination);
    // println!("Overwrite : {}, Exclude : {:?}",config.bgs.overwrite, config.bgs.exclude);

    let source_path = Path::new(&config.source);
    let destination_path = Path::new(&config.destination);

    // ソースフォルダが存在するか確認
    if !source_path.exists(){
        return Err(format!("Source folder does not exist : {}", config.source).into());
    }

    // ディレクトリを再帰的にコピー
    copy_recursive(source_path,destination_path,config.overwrite, &config.exclude)?;

    Ok(())
}

fn copy_recursive(
    source : &Path,
    destination : &Path,
    overwrite : bool,
    exclude : &[String],
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
            copy_recursive(&path, &destination, overwrite, exclude)?;
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
    }

    Ok(())


}

fn create_folders(config : &CgfConfig) -> Result<(),Box<dyn std::error::Error>> {
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
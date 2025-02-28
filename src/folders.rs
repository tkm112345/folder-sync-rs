use crate::config::CdfConfig;
use crate::messages::*;
use std::path::Path;
use std::fs;

pub fn execute_create_folders(config : &CdfConfig) -> Result<(),Box<dyn std::error::Error>> {
    // println!("Source : {}, Destination : {}",config.cgf.source, config.cgf.destination);
    let source_path = Path::new(&config.source);
    let destination_path = Path::new(&config.destination);

    // ソースフォルダが存在するか確認
    if !source_path.exists(){
        return Err(format!("{}",ERR_SOURCE_FOLDER_NOT_EXIST.replace("{}",&config.source)).into());
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


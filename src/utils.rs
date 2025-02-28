use indicatif::ProgressBar;
use log::info;
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::thread;

use crate::config::BtsConfig;

pub fn count_files(bts_configs: &[BtsConfig]) -> Result<u64, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
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
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            count += count_files_recursive(&path)?;
        }
    } else {
        count += 1;
    }
    Ok(count)
}

pub fn copy_recursive(
    source: &Path,
    destination: &Path,
    overwrite: bool,
    exclude: &[String],
    progress_bar: &Arc<ProgressBar>
) -> Result<(), Box<dyn std::error::Error>> {
    if exclude.iter().any(|e| source.ends_with(e)) {
        info!("Skipping excludes item : {}", source.display());
        return Ok(());
    }

    if source.is_dir() {
        std::fs::create_dir_all(destination)?;

        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let destination = destination.join(entry.file_name());
            copy_recursive(&path, &destination, overwrite, exclude, progress_bar)?;
        }
    } else {
        if destination.exists() {
            if !overwrite {
                info!("Skipping existing file: {}", destination.display());
                progress_bar.inc(1);
                return Ok(());
            }

            if let Ok(source_metadata) = std::fs::metadata(source) {
                if let Ok(destination_metadata) = std::fs::metadata(&destination) {
                    if source_metadata.len() == destination_metadata.len()
                        && source_metadata.modified()? == destination_metadata.modified()?
                        && source.file_name() == destination.file_name()
                    {
                        info!("Skipping unchanged file: {}", destination.display());
                        progress_bar.inc(1);
                        return Ok(());
                    }
                }
            }
        }

        std::fs::copy(source, destination)?;
        info!("Copied: {} to {}", source.display(), destination.display());
        progress_bar.inc(1);
    }

    Ok(())
}

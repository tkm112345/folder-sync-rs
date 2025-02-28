use crate::config::BtsConfig;
use crate::utils::{count_files, copy_recursive};
use indicatif::{ProgressBar, ProgressStyle};
use log::error;
use std::sync::Arc;
use std::thread;

pub fn execute_backup(bts_configs: &[BtsConfig]) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = vec![];
    let total_files = count_files(bts_configs)?;
    let progress_bar = Arc::new(ProgressBar::new(total_files));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")?
            .progress_chars("#>-"),
    );
    progress_bar.set_message("Backing up");

    for bts_config in bts_configs {
        let bts_config = Arc::new(bts_config.clone());
        let progress_bar = Arc::clone(&progress_bar);
        let handle = thread::spawn({
            let bts_config = Arc::clone(&bts_config);
            move || {
                if let Err(err) = backup_to_ssd(&bts_config, &progress_bar) {
                    error!("Backup failed : {}", err);
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    progress_bar.finish_with_message("Backup complete");

    Ok(())
}

fn backup_to_ssd(config: &BtsConfig, progress_bar: &Arc<ProgressBar>) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = std::path::Path::new(&config.source);
    let destination_path = std::path::Path::new(&config.destination);

    if !source_path.exists() {
        return Err(format!("Source folder does not exist : {}", config.source).into());
    }

    copy_recursive(source_path, destination_path, config.overwrite, &config.exclude, progress_bar)?;

    Ok(())
}

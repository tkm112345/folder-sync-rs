use crate::config::{BtsConfig,BtsConfigWrapper};
use crate::utils::{count_files, copy_recursive};
use crate::messages::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::error;
use std::sync::Arc;
use std::thread;

pub fn execute_backup(bts_config_wrapper: &BtsConfigWrapper) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = vec![];
    let total_files = count_files(&bts_config_wrapper.configs)?;
    let progress_bar = Arc::new(ProgressBar::new(total_files));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")?
            .progress_chars("#>-"),
    );
    progress_bar.set_message(MSG_BACKING_UP);

    for bts_config in &bts_config_wrapper.configs {
        let bts_config = Arc::new(bts_config.clone());
        let progress_bar = Arc::clone(&progress_bar);
        let exclude = bts_config_wrapper.exclude.clone();
        let handle = thread::spawn({
            let bts_config = Arc::clone(&bts_config);
            move || {
                if let Err(err) = backup_to_ssd(&bts_config, &exclude,&progress_bar) {
                    error!("Backup failed: {}", err);
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    progress_bar.finish_with_message(MSG_BACKUP_COMPLETE);

    Ok(())
}

fn backup_to_ssd(config: &BtsConfig,exclude : &[String], progress_bar: &Arc<ProgressBar>) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = std::path::Path::new(&config.source);
    let destination_path = std::path::Path::new(&config.destination);

    if !source_path.exists() {
        return Err(format!("Source folder does not exist : {}", config.source).into());
    }

    copy_recursive(source_path, destination_path, config.overwrite, exclude, progress_bar)?;

    Ok(())
}

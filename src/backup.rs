//! # Backup Module
//!
//! This module provides functionality for backing up files and directories based on specified configurations.
//! It utilizes multi-threading for concurrent backups and provides progress tracking.

use crate::config::{BtsConfig,BtsConfigWrapper};
use crate::utils::{count_files, copy_recursive};
use crate::messages::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::error;
use std::sync::Arc;
use std::thread;

/// Executes the backup process based on the provided configuration wrapper.
///
/// This function spawns multiple threads to perform backups concurrently. It calculates the total number of files to be backed up,
/// initializes a progress bar, and then iterates through the configurations to start individual backup threads.
///
/// # Arguments
///
/// * `bts_config_wrapper` - A reference to the `BtsConfigWrapper` struct containing backup configurations.
///
/// # Returns
///
/// Returns `Ok(())` if all backups are successful, or `Err(Box<dyn std::error::Error>)` if any backup fails.
///
/// # Errors
///
/// * Returns an error if counting files fails.
/// * Returns an error if progress bar style template is invalid.
/// * Returns an error if any of the backup threads fail.
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

/// Performs a backup to the specified destination based on the provided configuration.
///
/// This function copies files from the source to the destination, excluding specified files or directories.
/// It updates the progress bar during the copy process.
///
/// # Arguments
///
/// * `config` - A reference to the `BtsConfig` struct containing backup configuration.
/// * `exclude` - A slice of strings representing files or directories to exclude from the backup.
/// * `progress_bar` - An `Arc<ProgressBar>` for updating the backup progress.
///
/// # Returns
///
/// Returns `Ok(())` if the backup is successful, or `Err(Box<dyn std::error::Error>)` if an error occurs.
///
/// # Errors
///
/// * Returns an error if the source folder does not exist.
/// * Returns an error if the recursive copy operation fails.
fn backup_to_ssd(config: &BtsConfig,exclude : &[String], progress_bar: &Arc<ProgressBar>) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = std::path::Path::new(&config.source);
    let destination_path = std::path::Path::new(&config.destination);

    if !source_path.exists() {
        return Err(format!("Source folder does not exist : {}", config.source).into());
    }

    copy_recursive(source_path, destination_path, config.overwrite, exclude, progress_bar)?;

    Ok(())
}

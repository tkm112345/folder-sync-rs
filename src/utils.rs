//! # File Utility Module
//!
//! This module provides utility functions for counting files and recursively copying files and directories.
//! It utilizes multi-threading for efficient file counting and provides progress tracking during copying.

use indicatif::ProgressBar;
use log::info;
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::thread;

use crate::config::BtsConfig;

/// Counts the total number of files in the specified configurations.
///
/// This function spawns multiple threads to recursively count files in each source directory specified in the `bts_configs`.
/// It uses a channel to collect the counts from each thread and returns the total count.
///
/// # Arguments
///
/// * `bts_configs` - A slice of `BtsConfig` structs containing source directory configurations.
///
/// # Returns
///
/// Returns `Ok(u64)` with the total number of files, or `Err(Box<dyn std::error::Error>)` if an error occurs.
///
/// # Errors
///
/// * Returns an error if any of the file counting threads fail.
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

    drop(tx); // Drop the main thread's sender to signal completion
    
    let mut total_files = 0;
    for count in rx {
        total_files += count;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(total_files)
}

/// Recursively counts the number of files in a given path.
///
/// This function recursively traverses the directory structure and counts the number of files.
///
/// # Arguments
///
/// * `path` - A reference to the `Path` to count files in.
///
/// # Returns
///
/// Returns `Ok(u64)` with the number of files, or `Err(Box<dyn std::error::Error>)` if an error occurs.
///
/// # Errors
///
/// * Returns an error if reading the directory fails.
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

/// Recursively copies files and directories from source to destination.
///
/// This function recursively copies files and directories from the source path to the destination path.
/// It supports excluding specified files or directories, overwriting existing files, and tracking progress using a progress bar.
///
/// # Arguments
///
/// * `source` - A reference to the `Path` of the source.
/// * `destination` - A reference to the `Path` of the destination.
/// * `overwrite` - A boolean indicating whether to overwrite existing files.
/// * `exclude` - A slice of strings representing files or directories to exclude.
/// * `progress_bar` - An `Arc<ProgressBar>` for tracking the copy progress.
///
/// # Returns
///
/// Returns `Ok(())` if the copy is successful, or `Err(Box<dyn std::error::Error>)` if an error occurs.
///
/// # Errors
///
/// * Returns an error if directory creation fails.
/// * Returns an error if reading the directory fails.
/// * Returns an error if file copying fails.
/// * Returns an error if file metadata retrieval fails.
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

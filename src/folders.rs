//! # Folder Operation Module
//!
//! This module provides functionality for creating and recursively copying folders based on specified configurations.

use crate::config::CdfConfig;
use crate::messages::*;
use std::path::Path;
use std::fs;

/// Creates folders based on the given configuration.
///
/// Checks if the source folder specified in `config` exists.
/// If it exists, recursively creates the source folder structure in the destination folder.
///
/// # Arguments
///
/// * `config` - A reference to the `CdfConfig` struct containing folder creation settings.
///
/// # Returns
///
/// Returns `Ok(())` if successful, or `Err(Box<dyn std::error::Error>)` if an error occurs.
///
/// # Errors
///
/// * Returns an error if the source folder does not exist.
pub fn execute_create_folders(config : &CdfConfig) -> Result<(),Box<dyn std::error::Error>> {
    // println!("Source : {}, Destination : {}",config.cgf.source, config.cgf.destination);
    let source_path = Path::new(&config.source);
    let destination_path = Path::new(&config.destination);

    // Check if the source folder exists
    if !source_path.exists(){
        return Err(format!("{}",ERR_SOURCE_FOLDER_NOT_EXIST.replace("{}",&config.source)).into());
    }
    
    // Recursively create the folder structure
    create_folders_recursive(source_path, destination_path)?;

    Ok(())
}

/// Helper function to recursively create folders.
///
/// Recursively creates the folder structure specified by `source` in `destination`.
///
/// # Arguments
///
/// * `source` - A reference to the `Path` of the source folder.
/// * `destination` - A reference to the `Path` of the destination folder.
///
/// # Returns
///
/// Returns `Ok(())` if successful, or `Err(Box<dyn std::error::Error>)` if an error occurs.
///
/// # Errors
///
/// * Returns an error if directory creation fails.
/// * Returns an error if reading items in the directory fails.
fn create_folders_recursive(
    source:&Path,
    destination: &Path
) -> Result<(),Box<dyn std::error::Error>>{
    if source.is_dir() {

        // Skip if the folder already exists in the destination
        if !destination.exists(){
            // Create the directory
            fs::create_dir_all(destination)?;
        }

        // Recursively process items in the directory
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let destination = destination.join(entry.file_name());
            create_folders_recursive(&path,&destination)?;
        }
    }

    Ok(())
}


//! # Configuration Module
//!
//! This module defines structures for application configuration and provides functionality
//! to load configuration from a JSON file.

use crate::messages::*;
use serde::Deserialize;
use std::{io::Read,fs};
use std::path::Path;
/// Application configuration structure.
///
/// This struct represents the overall application configuration, including backup and folder creation settings.
#[derive(Deserialize)]
pub struct AppConfig {
    /// Backup configuration wrapper.
    pub bts : BtsConfigWrapper,
    /// Folder creation configuration.
    pub cdf : CdfConfig,
}

/// Backup configuration wrapper structure.
///
/// This struct wraps a vector of backup configurations and a list of excluded files or directories.
#[derive(Deserialize)]
pub struct BtsConfigWrapper {
    /// Vector of backup configurations.
    pub configs : Vec<BtsConfig>,
    /// Vector of excluded file or directory names.
    pub exclude : Vec<String>,
}

/// Backup configuration structure.
///
/// This struct represents the configuration for a single backup operation.
#[derive(Deserialize,Clone)]
pub struct BtsConfig {
    /// Source path for the backup.
    pub source : String,
    /// Destination path for the backup.
    pub destination : String,
    /// Flag indicating whether to overwrite existing files.
    pub overwrite: bool
}

/// Folder creation configuration structure.
///
/// This struct represents the configuration for creating folder structures.
#[derive(Deserialize)]
pub struct CdfConfig {
    /// Source path for the folder structure.
    pub source : String,
    /// Destination path for the folder structure.
    pub destination : String,
}

/// Loads application configuration from a JSON file.
///
/// This function reads the configuration file from the same directory as the executable,
/// parses it as JSON, and returns an `AppConfig` struct.
///
/// # Arguments
///
/// * `file_name` - The name of the configuration file.
///
/// # Returns
///
/// Returns `Ok(AppConfig)` if the configuration is loaded successfully,
/// or `Err(Box<dyn std::error::Error>)` if an error occurs.
///
/// # Errors
///
/// * Returns an error if the executable directory cannot be determined.
/// * Returns an error if the configuration file cannot be opened.
/// * Returns an error if the configuration file cannot be read.
/// * Returns an error if the configuration file cannot be parsed as JSON.
pub fn load_config(path : &Path) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: AppConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

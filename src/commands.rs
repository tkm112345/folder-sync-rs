//! # Command Line Interface Module
//!
//! This module defines the command-line interface for the application using the `clap` crate.
//! It provides structures and enums for parsing command-line arguments and subcommands.

// cspell:ignore PKGNAME

use clap::{Parser,Subcommand};

const VERSION :&str         = env!("CARGO_PKG_VERSION");
const PKGNAME: &str         = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str     = env!("CARGO_PKG_DESCRIPTION");


/// Command-line argument parser.
///
/// This struct defines the structure for parsing command-line arguments using the `clap` crate.
/// It includes subcommands and global options.
#[derive(Parser)]
#[clap(
    name = PKGNAME,
    about = DESCRIPTION,
    version = VERSION
)]
pub struct Cli {
    /// Subcommand to execute.
    #[clap(subcommand)]
    pub command : Commands,

    /// Enable verbose logging.
    #[clap(short,long,help = "詳細なログを出力")]
    pub verbose : bool,
}

/// Available subcommands.
///
/// This enum defines the available subcommands for the application.
#[derive(Subcommand)]
pub enum Commands {
    /// Backup to SSD.
    #[command(alias = "-bts",name = "--backup-to-ssd")]
    BackupToSsd,
    /// Create destination folder structure.
    #[command(alias = "-cdf",name = "--create-destination-folders")]
    CreateFolders,
}



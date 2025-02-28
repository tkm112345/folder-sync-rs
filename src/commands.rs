// cspell:ignore PKGNAME

use clap::{Parser,Subcommand};

const VERSION :&str = env!("CARGO_PKG_VERSION");
const PKGNAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");



#[derive(Parser)]
#[clap(
    name = PKGNAME,
    about = DESCRIPTION,
    version = VERSION
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command : Commands,

    #[clap(short,long,help = "詳細なログを出力")]
    pub verbose : bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// バックアップ
    #[command(alias = "-bts",name = "--backup-to-ssd")]
    BackupToSsd,
    /// フォルダ構成作成
    #[command(alias = "-cdf",name = "--create-destination-folders")]
    CreateFolders,
}



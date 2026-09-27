use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tagmv", version, about = "Organize music files by audio tags")]
pub struct Cli {
    /// Directory to sort (defaults to current directory)
    pub path: Option<PathBuf>,

    /// Preview changes without moving any files
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Skip the interactive approval prompt and move all planned files
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,

    /// Scan subdirectories
    #[arg(short, long)]
    pub recursive: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install file manager context menu integration
    Install,
    /// Remove file manager context menu integration
    Uninstall,
    /// Show whether the context menu integration is installed and enabled
    Status,
}

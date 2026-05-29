use anyhow::{Context, Result};
use clap::Parser;
use tagmv::cli::{Cli, Commands};
use tagmv::{install, run, RunOptions};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install) => return install::install_quick_action(),
        Some(Commands::Uninstall) => return install::uninstall_quick_action(),
        None => {}
    }

    let dir = match cli.path {
        Some(p) => p,
        None => std::env::current_dir()
            .context("Could not determine current directory. Please specify a path.")?,
    };

    run(RunOptions {
        dir,
        execute: !cli.dry_run,
        recursive: cli.recursive,
        assume_yes: cli.yes,
    })
}

#![warn(clippy::all)]

pub mod cli;
pub mod display;
pub mod install;
pub mod scan;
pub mod sorting;
pub mod tags;

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

use sorting::{
    compute_destination, compute_unsorted_destination, execute_move, resolve_conflicts, PlannedMove,
};
use tags::read_tags;

/// Options controlling a single sort run.
pub struct RunOptions {
    pub dir: PathBuf,
    pub execute: bool,
    pub recursive: bool,
}

/// Scan `dir`, read tags, compute destinations, and resolve conflicts.
/// Returns the fully planned moves without touching the filesystem.
pub fn plan_moves(dir: &Path, recursive: bool) -> Result<Vec<PlannedMove>> {
    let files = scan::scan_files(dir, recursive)?;

    let mut moves: Vec<PlannedMove> = Vec::with_capacity(files.len());
    for file in &files {
        let planned = match read_tags(file) {
            Some(meta) => compute_destination(dir, file, &meta),
            None => compute_unsorted_destination(dir, file),
        };
        moves.push(planned);
    }

    resolve_conflicts(&mut moves);
    Ok(moves)
}

/// Run a sort over `opts.dir`: print the preview and, unless this is a dry run,
/// execute the planned moves.
pub fn run(opts: RunOptions) -> Result<()> {
    let dir = std::fs::canonicalize(&opts.dir)
        .with_context(|| format!("Cannot resolve path: {}", opts.dir.display()))?;

    if !dir.is_dir() {
        anyhow::bail!("Not a directory: {}", dir.display());
    }

    let mode = if opts.execute {
        "EXECUTING"
    } else {
        "DRY RUN (no files will be moved)"
    };

    let version = env!("CARGO_PKG_VERSION");
    println!("tagmv v{} -- {}\n", version, mode.bold());
    println!("Scanning: {}", dir.display().to_string().dimmed());

    let moves = plan_moves(&dir, opts.recursive)?;
    println!("Found {} audio files\n", moves.len().to_string().bold());

    if moves.is_empty() {
        return Ok(());
    }

    let summary = display::preview(&moves);
    display::print_summary(&summary);

    if opts.execute {
        println!();
        let (success, errors) = execute_all(&moves);
        display::report(success, errors);
    }

    Ok(())
}

fn execute_all(moves: &[PlannedMove]) -> (u32, u32) {
    let mut success = 0u32;
    let mut errors = 0u32;

    for m in moves {
        if m.source == m.dest {
            continue;
        }

        match execute_move(m) {
            Ok(()) => success += 1,
            Err(e) => {
                eprintln!(
                    "  {} {} -> {}: {}",
                    "ERROR".red().bold(),
                    m.source.display(),
                    m.dest.display(),
                    e
                );
                errors += 1;
            }
        }
    }

    (success, errors)
}

mod install;
mod sorting;
mod tags;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use sorting::{
    compute_destination, compute_unsorted_destination, execute_move, resolve_conflicts,
    PlannedMove,
};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tags::read_tags;
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "m4a", "flac", "ogg", "wma", "aac", "wav"];

#[derive(Parser)]
#[command(name = "tagmv", version, about = "Organize music files by audio tags")]
struct Cli {
    /// Directory to sort (defaults to current directory)
    path: Option<PathBuf>,

    /// Actually move files (default is dry-run preview)
    #[arg(long)]
    execute: bool,

    /// Scan subdirectories
    #[arg(short, long)]
    recursive: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install macOS Finder Quick Action
    Install,
}

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

fn scan_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if recursive {
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                if e.file_type().is_dir() {
                    if e.depth() == 0 {
                        return true;
                    }
                    return !is_hidden(&name) && name != "_Unsorted";
                }
                true
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if path.is_file()
                && is_audio_file(&path)
                && !is_hidden(&entry.file_name().to_string_lossy())
            {
                files.push(path);
            }
        }
    } else {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && is_audio_file(&path) {
                let name = entry.file_name().to_string_lossy().to_string();
                if !is_hidden(&name) {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Install) = cli.command {
        return install::install_quick_action();
    }

    let dir = match cli.path {
        Some(p) => p,
        None => std::env::current_dir()
            .context("Could not determine current directory. Please specify a path.")?,
    };

    let dir = std::fs::canonicalize(&dir)
        .with_context(|| format!("Cannot resolve path: {}", dir.display()))?;

    if !dir.is_dir() {
        anyhow::bail!("Not a directory: {}", dir.display());
    }

    let mode = if cli.execute {
        "EXECUTING"
    } else {
        "DRY RUN (use --execute to move files)"
    };

    let version = env!("CARGO_PKG_VERSION");
    println!("tagmv v{} -- {}\n", version, mode.bold());
    println!("Scanning: {}", dir.display().to_string().dimmed());

    let files = scan_files(&dir, cli.recursive)?;
    println!("Found {} audio files\n", files.len().to_string().bold());

    if files.is_empty() {
        return Ok(());
    }

    let mut moves: Vec<PlannedMove> = Vec::new();

    for file in &files {
        match read_tags(file) {
            Some(meta) => {
                let planned = compute_destination(&dir, file, &meta);
                moves.push(planned);
            }
            None => {
                let planned = compute_unsorted_destination(&dir, file);
                moves.push(planned);
            }
        }
    }

    resolve_conflicts(&mut moves);

    // Group by folder for display
    let mut folders: BTreeMap<String, Vec<&PlannedMove>> = BTreeMap::new();
    for m in &moves {
        folders.entry(m.folder_name.clone()).or_default().push(m);
    }

    let mut move_count = 0u32;
    let mut unsorted_count = 0u32;
    let mut skipped_count = 0u32;

    for (folder, folder_moves) in &folders {
        if folder == "_Unsorted" {
            println!("  {}", folder.red().bold());
        } else {
            println!("  {}", format!("{}/", folder).yellow().bold());
        }

        for m in folder_moves {
            if m.source == m.dest {
                skipped_count += 1;
                println!(
                    "    {}  {}",
                    m.file_name.dimmed(),
                    "(already in place)".dimmed()
                );
            } else {
                let source_name = m
                    .source
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");

                println!(
                    "    {}  {} {}",
                    m.file_name.green(),
                    "<-".dimmed(),
                    source_name.dimmed()
                );

                if folder == "_Unsorted" {
                    unsorted_count += 1;
                } else {
                    move_count += 1;
                }
            }
        }

        println!();
    }

    let folder_count = folders.keys().filter(|k| *k != "_Unsorted").count();
    let total = move_count + unsorted_count + skipped_count;
    println!(
        "Summary: {} files -> {} folders, {} unsorted{}",
        total,
        folder_count,
        unsorted_count,
        if skipped_count > 0 {
            format!(", {} already in place", skipped_count)
        } else {
            String::new()
        }
    );

    if cli.execute {
        println!();
        let mut success = 0u32;
        let mut errors = 0u32;

        for m in &moves {
            if m.source == m.dest {
                continue;
            }

            match execute_move(m) {
                Ok(()) => {
                    success += 1;
                }
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

        println!(
            "Moved {} files successfully{}",
            success,
            if errors > 0 {
                format!(", {} errors", errors)
            } else {
                String::new()
            }
        );
    }

    Ok(())
}

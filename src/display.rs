use crate::sorting::PlannedMove;
use colored::Colorize;
use std::collections::BTreeMap;

/// Tally of what the planned moves will do, used for the summary line.
pub struct Summary {
    pub moved: u32,
    pub unsorted: u32,
    pub skipped: u32,
    pub folders: usize,
}

/// Print the grouped, colored preview of planned moves and return a tally.
pub fn preview(moves: &[PlannedMove]) -> Summary {
    let mut folders: BTreeMap<&str, Vec<&PlannedMove>> = BTreeMap::new();
    for m in moves {
        folders.entry(m.folder_name.as_str()).or_default().push(m);
    }

    let mut moved = 0u32;
    let mut unsorted = 0u32;
    let mut skipped = 0u32;

    for (folder, folder_moves) in &folders {
        if *folder == "_Unsorted" {
            println!("  {}", folder.red().bold());
        } else {
            println!("  {}", format!("{}/", folder).yellow().bold());
        }

        for m in folder_moves {
            if m.source == m.dest {
                skipped += 1;
                println!(
                    "    {}  {}",
                    m.file_name.dimmed(),
                    "(already in place)".dimmed()
                );
            } else {
                let source_name = m.source.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                println!(
                    "    {}  {} {}",
                    m.file_name.green(),
                    "<-".dimmed(),
                    source_name.dimmed()
                );

                if *folder == "_Unsorted" {
                    unsorted += 1;
                } else {
                    moved += 1;
                }
            }
        }

        println!();
    }

    let folder_count = folders.keys().filter(|k| **k != "_Unsorted").count();

    Summary {
        moved,
        unsorted,
        skipped,
        folders: folder_count,
    }
}

/// Print the summary line that follows the preview.
pub fn print_summary(summary: &Summary) {
    let total = summary.moved + summary.unsorted + summary.skipped;
    println!(
        "Summary: {} files -> {} folders, {} unsorted{}",
        total,
        summary.folders,
        summary.unsorted,
        if summary.skipped > 0 {
            format!(", {} already in place", summary.skipped)
        } else {
            String::new()
        }
    );
}

/// Print the result line after executing moves.
pub fn report(success: u32, errors: u32) {
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

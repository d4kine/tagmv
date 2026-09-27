use crate::sorting::PlannedMove;
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect};

/// Label a planned move for the selection list, e.g. `Artist - Album/01 - Title.ext`.
fn label(m: &PlannedMove) -> String {
    let source = m.source.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    format!("{}/{}  <- {}", m.folder_name, m.file_name, source)
}

/// Show an interactive checkbox list (everything pre-checked) and return the
/// subset of moves the user approved, or `None` when they aborted with Esc/q.
/// Deselecting an item skips that move.
pub fn select<'a>(moves: &[&'a PlannedMove]) -> Result<Option<Vec<&'a PlannedMove>>> {
    let items: Vec<String> = moves.iter().map(|m| label(m)).collect();
    let defaults = vec![true; moves.len()];

    let chosen = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Confirm moves (space toggles, a selects all, enter confirms, esc aborts)")
        .items(&items)
        .defaults(&defaults)
        .interact_opt()
        .inspect_err(|_| {
            // dialoguer leaves the cursor hidden when interrupted (Ctrl-C)
            let _ = dialoguer::console::Term::stderr().show_cursor();
        })
        .context("Approval prompt failed")?;

    Ok(chosen.map(|idx| idx.into_iter().map(|i| moves[i]).collect()))
}

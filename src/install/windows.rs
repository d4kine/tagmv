use super::{exe_path, warn_if_build_dir, MENU_LABEL};
use anyhow::{bail, Context, Result};

fn command_value(exe_str: &str) -> String {
    format!("\"{}\" \"%V\"", exe_str)
}

pub(super) fn install() -> Result<()> {
    let (exe, exe_str) = exe_path()?;
    warn_if_build_dir(&exe_str);

    let command_value = command_value(&exe_str);

    // Right-click on a folder
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\Directory\shell\tagmv",
        "/ve",
        "/d",
        MENU_LABEL,
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\Directory\shell\tagmv\command",
        "/ve",
        "/d",
        &command_value,
        "/f",
    ])?;

    // Right-click on folder background (inside a folder)
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\Directory\Background\shell\tagmv",
        "/ve",
        "/d",
        MENU_LABEL,
        "/f",
    ])?;
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\Directory\Background\shell\tagmv\command",
        "/ve",
        "/d",
        &command_value,
        "/f",
    ])?;

    println!(
        "Installed Windows Explorer context menu: \"{}\"",
        MENU_LABEL
    );
    println!("  Binary: {}", exe.display());
    println!();
    println!(
        "Usage: Right-click a folder in Explorer -> \"{}\"",
        MENU_LABEL
    );
    Ok(())
}

pub(super) fn uninstall() -> Result<()> {
    let keys = [
        r"HKCU\Software\Classes\Directory\shell\tagmv",
        r"HKCU\Software\Classes\Directory\Background\shell\tagmv",
    ];
    let mut removed = 0;
    for key in &keys {
        // /f = force (no prompt), failure is ok if key doesn't exist
        if run_reg(&["delete", key, "/f"]).is_ok() {
            println!("Removed: {}", key);
            removed += 1;
        }
    }
    if removed == 0 {
        println!("Nothing to remove (registry keys not found)");
    }
    Ok(())
}

pub(super) fn status() -> Result<()> {
    let key = r"HKCU\Software\Classes\Directory\shell\tagmv";
    if run_reg(&["query", key]).is_ok() {
        println!("  installed {}", key);
        Ok(())
    } else {
        bail!("Context menu not installed. Run: tagmv install")
    }
}

fn run_reg(args: &[&str]) -> Result<()> {
    let output = std::process::Command::new("reg")
        .args(args)
        .output()
        .context("Failed to run 'reg' command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("reg {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_value_format() {
        let exe = r"C:\Users\chris\bin\tagmv.exe";
        assert_eq!(command_value(exe), r#""C:\Users\chris\bin\tagmv.exe" "%V""#);
    }
}

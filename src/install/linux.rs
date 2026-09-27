use super::{exe_path, home_dir, shell_escape, warn_if_build_dir, MENU_LABEL};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

fn paths() -> Result<(PathBuf, PathBuf, PathBuf)> {
    let data = home_dir()?.join(".local/share");
    Ok((
        data.join("nautilus/scripts").join(MENU_LABEL),
        data.join("nemo/actions/tagmv.nemo_action"),
        data.join("kio/servicemenus/tagmv.desktop"),
    ))
}

fn nautilus_script(escaped_exe: &str) -> String {
    format!(
        "#!/bin/bash\nIFS=$'\\n'\nfor f in $NAUTILUS_SCRIPT_SELECTED_FILE_PATHS; do\n  [ -d \"$f\" ] && {} \"$f\"\ndone\n",
        escaped_exe
    )
}

fn nemo_action(exe_str: &str) -> String {
    format!(
        "[Nemo Action]\nName={}\nComment=Organize music files by audio tags\nExec={} %F\nIcon-Name=audio-x-generic\nSelection=Any\nExtensions=dir;\n",
        MENU_LABEL, exe_str
    )
}

fn dolphin_desktop(exe_str: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Service\nMimeType=inode/directory;\nActions=tagmv\n\n[Desktop Action tagmv]\nName={}\nExec={} %f\nIcon=audio-x-generic\n",
        MENU_LABEL, exe_str
    )
}

pub(super) fn install() -> Result<()> {
    let (exe, exe_str) = exe_path()?;
    warn_if_build_dir(&exe_str);

    let escaped = shell_escape(&exe_str);
    let (nautilus_path, nemo_path, dolphin_path) = paths()?;

    // --- Nautilus (GNOME Files) ---
    if let Some(parent) = nautilus_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&nautilus_path, nautilus_script(&escaped))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&nautilus_path, fs::Permissions::from_mode(0o755))?;
    }
    println!("  Nautilus: {}", nautilus_path.display());

    // --- Nemo (Cinnamon) ---
    if let Some(parent) = nemo_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&nemo_path, nemo_action(&exe_str))?;
    println!("  Nemo:     {}", nemo_path.display());

    // --- Dolphin (KDE) ---
    if let Some(parent) = dolphin_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&dolphin_path, dolphin_desktop(&exe_str))?;
    println!("  Dolphin:  {}", dolphin_path.display());

    println!();
    println!("Installed context menu for Nautilus, Nemo, and Dolphin.");
    println!("  Binary: {}", exe.display());
    println!();
    println!(
        "Usage: Right-click a folder -> Scripts/Actions -> \"{}\"",
        MENU_LABEL
    );
    Ok(())
}

pub(super) fn uninstall() -> Result<()> {
    let (nautilus_path, nemo_path, dolphin_path) = paths()?;
    let mut removed = 0;
    for path in [&nautilus_path, &nemo_path, &dolphin_path] {
        if path.exists() {
            fs::remove_file(path)?;
            println!("Removed: {}", path.display());
            removed += 1;
        }
    }
    if removed == 0 {
        println!("Nothing to remove (no integrations found)");
    }
    Ok(())
}

pub(super) fn status() -> Result<()> {
    let (nautilus_path, nemo_path, dolphin_path) = paths()?;
    let mut installed = 0;
    for path in [&nautilus_path, &nemo_path, &dolphin_path] {
        let mark = if path.exists() {
            installed += 1;
            "installed"
        } else {
            "missing"
        };
        println!("  {:9} {}", mark, path.display());
    }
    if installed == 0 {
        anyhow::bail!("Context menu not installed. Run: tagmv install");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nautilus_script_content() {
        let script = nautilus_script(&shell_escape("/usr/local/bin/tagmv"));
        assert!(script.starts_with("#!/bin/bash"));
        assert!(script.contains("'/usr/local/bin/tagmv'"));
        assert!(script.contains("NAUTILUS_SCRIPT_SELECTED_FILE_PATHS"));
        assert!(!script.contains("--execute"));
    }

    #[test]
    fn nemo_action_content() {
        let action = nemo_action("/usr/local/bin/tagmv");
        assert!(action.contains("[Nemo Action]"));
        assert!(action.contains("Name=tagmv\n"));
        assert!(action.contains("Exec=/usr/local/bin/tagmv %F"));
        assert!(!action.contains("--execute"));
    }

    #[test]
    fn dolphin_desktop_content() {
        let desktop = dolphin_desktop("/usr/local/bin/tagmv");
        assert!(desktop.contains("Type=Service"));
        assert!(desktop.contains("inode/directory"));
        assert!(desktop.contains("[Desktop Action tagmv]"));
        assert!(desktop.contains("Exec=/usr/local/bin/tagmv %f"));
        assert!(!desktop.contains("--execute"));
    }
}

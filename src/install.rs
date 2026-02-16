use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

const MENU_LABEL: &str = "Sort Music by Tags";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn home_dir() -> Result<PathBuf> {
    // Try $HOME first (works on macOS, Linux, and sometimes Windows)
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(&home);
        if p.is_absolute() {
            return Ok(p);
        }
    }
    // Windows fallback
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(&profile);
        if p.is_absolute() {
            return Ok(p);
        }
    }
    bail!("Could not determine home directory ($HOME / %USERPROFILE% not set)")
}

fn exe_path() -> Result<(PathBuf, String)> {
    let exe = std::env::current_exe().context("Failed to determine current executable path")?;
    let exe_str = exe.to_string_lossy().to_string();
    Ok((exe, exe_str))
}

fn warn_if_build_dir(exe_str: &str) {
    let in_target = exe_str.contains("/target/") || exe_str.contains("\\target\\");
    if in_target {
        eprintln!("Warning: You are running from a build directory.");
        eprintln!("Consider copying the binary to a stable location first:");
        if cfg!(target_os = "windows") {
            eprintln!("  copy {} %USERPROFILE%\\bin\\tagmv.exe", exe_str);
            eprintln!("  %USERPROFILE%\\bin\\tagmv.exe install");
        } else {
            eprintln!("  cp {} ~/bin/tagmv", exe_str);
            eprintln!("  ~/bin/tagmv install");
        }
        eprintln!();
    }
}

/// Escape a string for safe embedding in XML text content.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Shell-escape a path for embedding in a single-quoted sh/zsh string.
fn shell_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub fn install_quick_action() -> Result<()> {
    if cfg!(target_os = "macos") {
        install_macos()
    } else if cfg!(target_os = "linux") {
        install_linux()
    } else if cfg!(target_os = "windows") {
        install_windows()
    } else {
        bail!("Unsupported platform for context menu installation")
    }
}

pub fn uninstall_quick_action() -> Result<()> {
    if cfg!(target_os = "macos") {
        uninstall_macos()
    } else if cfg!(target_os = "linux") {
        uninstall_linux()
    } else if cfg!(target_os = "windows") {
        uninstall_windows()
    } else {
        bail!("Unsupported platform for context menu removal")
    }
}

// ===========================================================================
// macOS -- Automator Quick Action
// ===========================================================================

fn macos_workflow_dir() -> Result<PathBuf> {
    Ok(home_dir()?
        .join("Library/Services")
        .join(format!("{}.workflow", MENU_LABEL)))
}

fn install_macos() -> Result<()> {
    let (exe, exe_str) = exe_path()?;
    warn_if_build_dir(&exe_str);

    let wf_dir = macos_workflow_dir()?;
    let contents_dir = wf_dir.join("Contents");

    if wf_dir.exists() {
        fs::remove_dir_all(&wf_dir).with_context(|| {
            format!("Failed to remove existing workflow at {}", wf_dir.display())
        })?;
    }

    fs::create_dir_all(&contents_dir)
        .with_context(|| format!("Failed to create {}", contents_dir.display()))?;

    fs::write(contents_dir.join("document.wflow"), macos_document_wflow(&exe_str))
        .context("Failed to write document.wflow")?;
    fs::write(contents_dir.join("Info.plist"), macos_info_plist())
        .context("Failed to write Info.plist")?;

    println!("Installed macOS Quick Action: \"{}\"", MENU_LABEL);
    println!("  Location: {}", wf_dir.display());
    println!("  Binary:   {}", exe.display());
    println!();
    println!("Next steps:");
    println!("  1. Open System Settings -> Privacy & Security -> Extensions -> Finder");
    println!("  2. Enable \"{}\"", MENU_LABEL);
    println!("  3. If it doesn't appear, run: killall Finder");
    println!();
    println!("Usage: Right-click a folder in Finder -> Quick Actions -> \"{}\"", MENU_LABEL);
    Ok(())
}

fn uninstall_macos() -> Result<()> {
    let wf_dir = macos_workflow_dir()?;
    if wf_dir.exists() {
        fs::remove_dir_all(&wf_dir)?;
        println!("Removed: {}", wf_dir.display());
    } else {
        println!("Nothing to remove (workflow not found)");
    }
    Ok(())
}

fn macos_document_wflow(binary_path: &str) -> String {
    let shell_safe = shell_escape(binary_path);
    let xml_safe_script = xml_escape(&format!(
        "for f in \"$@\"; do\n  if [ -d \"$f\" ]; then\n    {} --execute \"$f\"\n  fi\ndone",
        shell_safe
    ));

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key>
	<string>528</string>
	<key>AMApplicationVersion</key>
	<string>2.10</string>
	<key>AMDocumentVersion</key>
	<string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Optional</key>
					<true/>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>AMActionVersion</key>
				<string>2.0.3</string>
				<key>AMApplication</key>
				<array>
					<string>Automator</string>
				</array>
				<key>AMParameterProperties</key>
				<dict>
					<key>COMMAND_STRING</key>
					<dict/>
					<key>CheckedForUserDefaultShell</key>
					<dict/>
					<key>inputMethod</key>
					<dict/>
					<key>shell</key>
					<dict/>
					<key>source</key>
					<dict/>
				</dict>
				<key>AMProvides</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>ActionBundlePath</key>
				<string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key>
				<string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key>
					<string>{script}</string>
					<key>CheckedForUserDefaultShell</key>
					<true/>
					<key>inputMethod</key>
					<integer>1</integer>
					<key>shell</key>
					<string>/bin/zsh</string>
					<key>source</key>
					<string></string>
				</dict>
				<key>BundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key>
				<string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key>
				<false/>
				<key>CanShowWhenRun</key>
				<true/>
				<key>Category</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>Class Name</key>
				<string>RunShellScriptAction</string>
				<key>InputUUID</key>
				<string>A1B2C3D4-E5F6-7890-ABCD-EF1234567890</string>
				<key>Keywords</key>
				<array>
					<string>Shell</string>
					<string>Script</string>
					<string>Command</string>
					<string>Run</string>
					<string>Unix</string>
				</array>
				<key>OutputUUID</key>
				<string>B2C3D4E5-F6A7-8901-BCDE-F12345678901</string>
				<key>UUID</key>
				<string>C3D4E5F6-A7B8-9012-CDEF-123456789012</string>
				<key>UnlocalizedApplications</key>
				<array>
					<string>Automator</string>
				</array>
				<key>arguments</key>
				<dict>
					<key>0</key>
					<dict>
						<key>default value</key>
						<integer>0</integer>
						<key>name</key>
						<string>inputMethod</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>0</string>
					</dict>
					<key>1</key>
					<dict>
						<key>default value</key>
						<false/>
						<key>name</key>
						<string>CheckedForUserDefaultShell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>1</string>
					</dict>
					<key>2</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>source</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>2</string>
					</dict>
					<key>3</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>COMMAND_STRING</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>3</string>
					</dict>
					<key>4</key>
					<dict>
						<key>default value</key>
						<string>/bin/sh</string>
						<key>name</key>
						<string>shell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>4</string>
					</dict>
				</dict>
				<key>isViewVisible</key>
				<integer>1</integer>
				<key>location</key>
				<string>354.500000:305.000000</string>
				<key>nibPath</key>
				<string>/System/Library/Automator/Run Shell Script.action/Contents/Resources/Base.lproj/main.nib</string>
			</dict>
			<key>isViewVisible</key>
			<integer>1</integer>
		</dict>
	</array>
	<key>connectors</key>
	<dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>applicationBundleID</key>
		<string>com.apple.finder</string>
		<key>applicationBundleIDsByPath</key>
		<dict>
			<key>/System/Library/CoreServices/Finder.app</key>
			<string>com.apple.finder</string>
		</dict>
		<key>applicationPath</key>
		<string>/System/Library/CoreServices/Finder.app</string>
		<key>applicationPaths</key>
		<array>
			<string>/System/Library/CoreServices/Finder.app</string>
		</array>
		<key>backgroundColorName</key>
		<string>blackColor</string>
		<key>inputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject</string>
		<key>outputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>presentationMode</key>
		<integer>15</integer>
		<key>processesInput</key>
		<false/>
		<key>serviceApplicationBundleID</key>
		<string>com.apple.finder</string>
		<key>serviceApplicationPath</key>
		<string>/System/Library/CoreServices/Finder.app</string>
		<key>serviceInputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject</string>
		<key>serviceOutputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key>
		<false/>
		<key>systemImageName</key>
		<string>NSTouchBarTagIcon</string>
		<key>useAutomaticInputType</key>
		<false/>
		<key>workflowTypeIdentifier</key>
		<string>com.apple.Automator.servicesMenu</string>
	</dict>
</dict>
</plist>"#,
        script = xml_safe_script
    )
}

fn macos_info_plist() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSServices</key>
	<array>
		<dict>
			<key>NSBackgroundColorName</key>
			<string>background</string>
			<key>NSBackgroundSystemColorName</key>
			<string>blackColor</string>
			<key>NSIconName</key>
			<string>NSTouchBarTagIcon</string>
			<key>NSMenuItem</key>
			<dict>
				<key>default</key>
				<string>Sort Music by Tags</string>
			</dict>
			<key>NSMessage</key>
			<string>runWorkflowAsService</string>
			<key>NSRequiredContext</key>
			<dict>
				<key>NSApplicationIdentifier</key>
				<string>com.apple.finder</string>
			</dict>
			<key>NSSendFileTypes</key>
			<array>
				<string>public.item</string>
			</array>
		</dict>
	</array>
</dict>
</plist>"#
}

// ===========================================================================
// Linux -- Nautilus script + Nemo action + Dolphin service menu
// ===========================================================================

fn linux_paths() -> Result<(PathBuf, PathBuf, PathBuf)> {
    let data = home_dir()?.join(".local/share");
    Ok((
        data.join("nautilus/scripts").join(MENU_LABEL),
        data.join("nemo/actions/tagmv.nemo_action"),
        data.join("kio/servicemenus/tagmv.desktop"),
    ))
}

fn install_linux() -> Result<()> {
    let (exe, exe_str) = exe_path()?;
    warn_if_build_dir(&exe_str);

    let escaped = shell_escape(&exe_str);
    let (nautilus_path, nemo_path, dolphin_path) = linux_paths()?;

    // --- Nautilus (GNOME Files) ---
    if let Some(parent) = nautilus_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let nautilus_script = format!(
        "#!/bin/bash\nIFS=$'\\n'\nfor f in $NAUTILUS_SCRIPT_SELECTED_FILE_PATHS; do\n  [ -d \"$f\" ] && {} --execute \"$f\"\ndone\n",
        escaped
    );
    fs::write(&nautilus_path, &nautilus_script)?;
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
    let nemo_action = format!(
        "[Nemo Action]\nName={}\nComment=Organize music files by audio tags\nExec={} --execute %F\nIcon-Name=audio-x-generic\nSelection=Any\nExtensions=dir;\n",
        MENU_LABEL, exe_str
    );
    fs::write(&nemo_path, &nemo_action)?;
    println!("  Nemo:     {}", nemo_path.display());

    // --- Dolphin (KDE) ---
    if let Some(parent) = dolphin_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let dolphin_desktop = format!(
        "[Desktop Entry]\nType=Service\nMimeType=inode/directory;\nActions=tagmv\n\n[Desktop Action tagmv]\nName={}\nExec={} --execute %f\nIcon=audio-x-generic\n",
        MENU_LABEL, exe_str
    );
    fs::write(&dolphin_path, &dolphin_desktop)?;
    println!("  Dolphin:  {}", dolphin_path.display());

    println!();
    println!("Installed context menu for Nautilus, Nemo, and Dolphin.");
    println!("  Binary: {}", exe.display());
    println!();
    println!("Usage: Right-click a folder -> Scripts/Actions -> \"{}\"", MENU_LABEL);
    Ok(())
}

fn uninstall_linux() -> Result<()> {
    let (nautilus_path, nemo_path, dolphin_path) = linux_paths()?;
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

// ===========================================================================
// Windows -- Explorer context menu via registry
// ===========================================================================

fn install_windows() -> Result<()> {
    let (exe, exe_str) = exe_path()?;
    warn_if_build_dir(&exe_str);

    let command_value = format!("\"{}\" --execute \"%V\"", exe_str);

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

    println!("Installed Windows Explorer context menu: \"{}\"", MENU_LABEL);
    println!("  Binary: {}", exe.display());
    println!();
    println!("Usage: Right-click a folder in Explorer -> \"{}\"", MENU_LABEL);
    Ok(())
}

fn uninstall_windows() -> Result<()> {
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

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escape_special_chars() {
        assert_eq!(xml_escape("a&b"), "a&amp;b");
        assert_eq!(xml_escape("a<b>c"), "a&lt;b&gt;c");
        assert_eq!(xml_escape("a\"b'c"), "a&quot;b&apos;c");
        assert_eq!(xml_escape("normal"), "normal");
    }

    #[test]
    fn shell_escape_simple_path() {
        assert_eq!(
            shell_escape("/usr/local/bin/tagmv"),
            "'/usr/local/bin/tagmv'"
        );
    }

    #[test]
    fn shell_escape_path_with_single_quote() {
        assert_eq!(shell_escape("/path/it's/here"), "'/path/it'\"'\"'s/here'");
    }

    #[test]
    fn shell_escape_path_with_spaces() {
        assert_eq!(shell_escape("/my path/bin"), "'/my path/bin'");
    }

    #[test]
    fn shell_escape_path_with_dollar() {
        assert_eq!(shell_escape("/path/$HOME/bin"), "'/path/$HOME/bin'");
    }

    #[test]
    fn home_dir_returns_absolute() {
        if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
            let home = home_dir().unwrap();
            assert!(home.is_absolute());
        }
    }

    #[test]
    fn linux_nautilus_script_content() {
        let escaped = shell_escape("/usr/local/bin/tagmv");
        let script = format!(
            "#!/bin/bash\nIFS=$'\\n'\nfor f in $NAUTILUS_SCRIPT_SELECTED_FILE_PATHS; do\n  [ -d \"$f\" ] && {} --execute \"$f\"\ndone\n",
            escaped
        );
        assert!(script.starts_with("#!/bin/bash"));
        assert!(script.contains("'/usr/local/bin/tagmv'"));
        assert!(script.contains("NAUTILUS_SCRIPT_SELECTED_FILE_PATHS"));
    }

    #[test]
    fn linux_nemo_action_content() {
        let action = format!(
            "[Nemo Action]\nName={}\nExec={} --execute %F\n",
            MENU_LABEL, "/usr/local/bin/tagmv"
        );
        assert!(action.contains("[Nemo Action]"));
        assert!(action.contains("Sort Music by Tags"));
        assert!(action.contains("--execute %F"));
    }

    #[test]
    fn linux_dolphin_desktop_content() {
        let desktop = format!(
            "[Desktop Entry]\nType=Service\nMimeType=inode/directory;\nActions=tagmv\n\n[Desktop Action tagmv]\nName={}\nExec={} --execute %f\n",
            MENU_LABEL, "/usr/local/bin/tagmv"
        );
        assert!(desktop.contains("Type=Service"));
        assert!(desktop.contains("inode/directory"));
        assert!(desktop.contains("[Desktop Action tagmv]"));
    }

    #[test]
    fn windows_command_value_format() {
        let exe = r"C:\Users\chris\bin\tagmv.exe";
        let cmd = format!("\"{}\" --execute \"%V\"", exe);
        assert_eq!(cmd, r#""C:\Users\chris\bin\tagmv.exe" --execute "%V""#);
    }
}

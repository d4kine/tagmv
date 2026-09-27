use super::{exe_path, home_dir, shell_escape, warn_if_build_dir, xml_escape, MENU_LABEL};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workflow_dir() -> Result<PathBuf> {
    Ok(home_dir()?
        .join("Library/Services")
        .join(format!("{}.workflow", MENU_LABEL)))
}

pub(super) fn install() -> Result<()> {
    let (exe, exe_str) = exe_path()?;
    warn_if_build_dir(&exe_str);

    let wf_dir = workflow_dir()?;
    let contents_dir = wf_dir.join("Contents");

    if wf_dir.exists() {
        fs::remove_dir_all(&wf_dir).with_context(|| {
            format!("Failed to remove existing workflow at {}", wf_dir.display())
        })?;
    }

    fs::create_dir_all(&contents_dir)
        .with_context(|| format!("Failed to create {}", contents_dir.display()))?;

    fs::write(
        contents_dir.join("document.wflow"),
        document_wflow(&shell_escape(&exe_str)),
    )
    .context("Failed to write document.wflow")?;
    fs::write(contents_dir.join("Info.plist"), info_plist())
        .context("Failed to write Info.plist")?;

    // Best effort: re-scan ~/Library/Services so the action shows up without a Finder restart.
    let _ = Command::new("/System/Library/CoreServices/pbs")
        .arg("-update")
        .status();

    println!("Installed macOS Quick Action: \"{}\"", MENU_LABEL);
    println!("  Location: {}", wf_dir.display());
    println!("  Binary:   {}", exe.display());
    println!();
    println!("Status:");
    status()?;
    println!();
    println!("If the action does not show up in Finder, run: killall Finder");
    println!();
    println!(
        "Usage: Right-click a folder (or files inside it) in Finder -> Quick Actions -> \"{}\"",
        MENU_LABEL
    );
    println!(
        "Files are moved immediately; the result is shown as a notification, errors as a dialog."
    );
    Ok(())
}

/// Enabled state as recorded by pbs (System Settings -> Extensions -> Finder).
#[derive(Debug, PartialEq)]
enum Enabled {
    Yes,
    No,
    /// pbs has not picked up the workflow yet (no entry).
    Unknown,
}

/// Parse `defaults read pbs NSServicesStatus` output. A missing
/// `enabled_context_menu` key means "enabled" (macOS default).
fn parse_enabled(services_status: &str) -> Enabled {
    let key = format!("- {} - runWorkflowAsService\"", MENU_LABEL);
    let Some(start) = services_status.find(&key) else {
        return Enabled::Unknown;
    };
    // The entry's dict closes with `};` at the same indentation as its key line.
    let line_start = services_status[..start].rfind('\n').map_or(0, |i| i + 1);
    let indent: String = services_status[line_start..]
        .chars()
        .take_while(|c| *c == ' ')
        .collect();
    let rest = &services_status[start + key.len()..];
    let closer = format!("\n{}}};", indent);
    let block = rest.find(&closer).map_or(rest, |end| &rest[..end]);
    if block.contains("\"enabled_context_menu\" = 0") {
        Enabled::No
    } else {
        Enabled::Yes
    }
}

fn enabled_state() -> Enabled {
    Command::new("defaults")
        .args(["read", "pbs", "NSServicesStatus"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| parse_enabled(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or(Enabled::Unknown)
}

pub(super) fn status() -> Result<()> {
    let wf_dir = workflow_dir()?;
    if !wf_dir.exists() {
        anyhow::bail!(
            "Quick Action not installed ({} missing). Run: tagmv install",
            wf_dir.display()
        );
    }
    println!("  installed {}", wf_dir.display());
    match enabled_state() {
        Enabled::Yes => println!("  enabled   in System Settings -> Extensions -> Finder"),
        Enabled::No => {
            println!("  DISABLED  in System Settings -> Extensions -> Finder");
            println!("            open x-apple.systempreferences:com.apple.ExtensionsPreferences");
        }
        Enabled::Unknown => {
            println!("  pending   not yet registered by macOS; run: /System/Library/CoreServices/pbs -update");
        }
    }
    Ok(())
}

pub(super) fn uninstall() -> Result<()> {
    let wf_dir = workflow_dir()?;
    if wf_dir.exists() {
        fs::remove_dir_all(&wf_dir)?;
        println!("Removed: {}", wf_dir.display());
    } else {
        println!("Nothing to remove (workflow not found)");
    }
    Ok(())
}

/// zsh script run by the Quick Action. Runs `tagmv -y` per selected folder
/// (files map to their parent folder, each folder once) and reports through
/// `display notification` / `display dialog`. All dynamic values reach
/// osascript as argv, never interpolated into AppleScript source.
/// `tool` is inserted verbatim after `tool=`, so it must already be shell-safe.
fn quick_action_script(tool: &str) -> String {
    format!(
        r#"tool={tool}
notify() {{ /usr/bin/osascript -e 'on run argv' -e 'display notification (item 2 of argv) with title "{label}" subtitle (item 1 of argv)' -e 'end run' "$1" "$2"; }}
fail() {{ /usr/bin/osascript -e 'on run argv' -e 'display dialog (item 2 of argv) with title "{label}" buttons {{"OK"}} default button "OK" with icon stop' -e 'end run' "$1" "$2" >/dev/null; }}

if [[ ! -x "$tool" ]]; then
  fail "" "tagmv not found or not executable: $tool"
  exit 0
fi

typeset -A seen
for item in "$@"; do
  if [[ -d "$item" ]]; then
    dir="$item"
  elif [[ -f "$item" ]]; then
    dir="${{item:h}}"
  else
    continue
  fi
  [[ -n ${{seen[$dir]-}} ]] && continue
  seen[$dir]=1

  out=$("$tool" -y -- "$dir" 2>&1); rc=$?
  if (( rc == 0 )); then
    notify "${{dir:t}}" "${{${{(f)out}}[-1]}}"
  else
    fail "${{dir:t}}" "${{dir}}"$'\n\n'"${{out[-1500,-1]}}"
  fi
done"#,
        tool = tool,
        label = MENU_LABEL,
    )
}

fn document_wflow(tool: &str) -> String {
    let xml_safe_script = xml_escape(&quick_action_script(tool));

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

fn info_plist() -> String {
    format!(
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
				<string>{label}</string>
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
</plist>"#,
        label = MENU_LABEL
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_action_script_runs_tagmv_non_interactively() {
        let script = quick_action_script(&shell_escape("/opt/it's/tagmv"));
        assert!(script.starts_with("tool='/opt/it'\"'\"'s/tagmv'\n"));
        assert!(script.contains("\"$tool\" -y -- \"$dir\""));
        assert!(script.contains("display notification"));
        assert!(script.contains("display dialog"));
        assert!(script.contains("dir=\"${item:h}\""));
    }

    #[test]
    fn document_wflow_is_xml_escaped() {
        let wflow = document_wflow(&shell_escape("/usr/local/bin/tagmv"));
        assert!(wflow.contains("&amp;&amp; continue"));
        assert!(wflow.contains("&quot;$tool&quot; -y -- &quot;$dir&quot;"));
        assert!(wflow.contains("<string>tool=&apos;/usr/local/bin/tagmv&apos;"));
    }

    #[test]
    fn parse_enabled_states() {
        let on = "{\n    \"(null) - tagmv - runWorkflowAsService\" = {\n        \"presentation_modes\" = { ContextMenu = 1; };\n    };\n    \"com.apple.Safari - Search - x\" = {\n        \"enabled_context_menu\" = 0;\n    };\n}";
        assert_eq!(parse_enabled(on), Enabled::Yes);
        let off = "{\n    \"(null) - tagmv - runWorkflowAsService\" = {\n        \"enabled_context_menu\" = 0;\n        \"enabled_services_menu\" = 0;\n    };\n}";
        assert_eq!(parse_enabled(off), Enabled::No);
        assert_eq!(parse_enabled("{\n}"), Enabled::Unknown);
    }

    /// Tool expression for the checked-in `contrib/` workflow: resolve `tagmv`
    /// from PATH at run time, falling back to `~/bin/tagmv`.
    const PORTABLE_TOOL: &str =
        r#""$(command -v tagmv 2>/dev/null || print -r -- "$HOME/bin/tagmv")""#;

    /// The checked-in workflow bundle must match the generator.
    /// Regenerate with: UPDATE_WORKFLOW=1 cargo test contrib_workflow
    #[test]
    fn contrib_workflow_matches_generator() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("contrib")
            .join(format!("{}.workflow", MENU_LABEL))
            .join("Contents");
        let expected = [
            ("document.wflow", document_wflow(PORTABLE_TOOL)),
            ("Info.plist", info_plist()),
        ];
        if std::env::var_os("UPDATE_WORKFLOW").is_some() {
            fs::create_dir_all(&root).unwrap();
            for (name, content) in &expected {
                fs::write(root.join(name), content).unwrap();
            }
        }
        for (name, content) in &expected {
            let on_disk = fs::read_to_string(root.join(name))
                .unwrap_or_else(|e| panic!("{}: {e} (run with UPDATE_WORKFLOW=1)", name));
            assert_eq!(
                &on_disk, content,
                "{} is stale, run with UPDATE_WORKFLOW=1",
                name
            );
        }
    }
}

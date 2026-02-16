use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const WORKFLOW_NAME: &str = "Sort Music by Tags";

fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .ok()
        .filter(|p| p.is_absolute())
        .context("$HOME is not set or is not an absolute path")
}

fn workflow_dir() -> Result<PathBuf> {
    Ok(home_dir()?
        .join("Library/Services")
        .join(format!("{}.workflow", WORKFLOW_NAME)))
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

/// Shell-escape a path for embedding in a single-quoted zsh string.
/// Single quotes cannot appear inside single-quoted strings in sh/zsh,
/// so we break out and splice them as double-quoted.
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

fn document_wflow(binary_path: &str) -> String {
    let shell_safe = shell_escape(binary_path);
    // The shell script is embedded inside XML, so the shell-escaped path
    // also needs XML escaping for the plist.
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

fn info_plist() -> &'static str {
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

pub fn install_quick_action() -> Result<()> {
    let exe = std::env::current_exe().context("Failed to determine current executable path")?;
    let exe_str = exe.to_string_lossy();

    // Warn if running from a build directory
    if exe_str.contains("/target/") {
        eprintln!("Warning: You are running from a build directory.");
        eprintln!("Consider copying the binary to a stable location first:");
        eprintln!("  cp {} ~/bin/tagmv", exe.display());
        eprintln!("  ~/bin/tagmv install");
        eprintln!();
    }

    let wf_dir = workflow_dir()?;
    let contents_dir = wf_dir.join("Contents");

    // Remove existing workflow if present
    if wf_dir.exists() {
        fs::remove_dir_all(&wf_dir).with_context(|| {
            format!(
                "Failed to remove existing workflow at {}",
                wf_dir.display()
            )
        })?;
    }

    fs::create_dir_all(&contents_dir)
        .with_context(|| format!("Failed to create {}", contents_dir.display()))?;

    fs::write(
        contents_dir.join("document.wflow"),
        document_wflow(&exe_str),
    )
    .context("Failed to write document.wflow")?;

    fs::write(contents_dir.join("Info.plist"), info_plist())
        .context("Failed to write Info.plist")?;

    println!("Installed Quick Action: \"{}\"", WORKFLOW_NAME);
    println!("  Location: {}", wf_dir.display());
    println!("  Binary:   {}", exe.display());
    println!();
    println!("Next steps:");
    println!("  1. Open System Settings -> Privacy & Security -> Extensions -> Finder");
    println!("  2. Enable \"{}\"", WORKFLOW_NAME);
    println!("  3. If it doesn't appear, run: killall Finder");
    println!();
    println!(
        "Usage: Right-click a folder in Finder -> Quick Actions -> \"{}\"",
        WORKFLOW_NAME
    );

    Ok(())
}

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
        assert_eq!(shell_escape("/usr/local/bin/musicsort"), "'/usr/local/bin/musicsort'");
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
        // Single-quoted strings don't expand $
        assert_eq!(shell_escape("/path/$HOME/bin"), "'/path/$HOME/bin'");
    }

    #[test]
    fn home_dir_returns_absolute() {
        // This test relies on $HOME being set in the test environment
        if std::env::var("HOME").is_ok() {
            let home = home_dir().unwrap();
            assert!(home.is_absolute());
        }
    }
}

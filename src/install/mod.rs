use anyhow::{bail, Context, Result};
use std::path::PathBuf;

mod linux;
mod macos;
mod windows;

pub(crate) const MENU_LABEL: &str = "Sort Music by Tags";

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub fn install_quick_action() -> Result<()> {
    if cfg!(target_os = "macos") {
        macos::install()
    } else if cfg!(target_os = "linux") {
        linux::install()
    } else if cfg!(target_os = "windows") {
        windows::install()
    } else {
        bail!("Unsupported platform for context menu installation")
    }
}

pub fn uninstall_quick_action() -> Result<()> {
    if cfg!(target_os = "macos") {
        macos::uninstall()
    } else if cfg!(target_os = "linux") {
        linux::uninstall()
    } else if cfg!(target_os = "windows") {
        windows::uninstall()
    } else {
        bail!("Unsupported platform for context menu removal")
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

pub(crate) fn home_dir() -> Result<PathBuf> {
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

pub(crate) fn exe_path() -> Result<(PathBuf, String)> {
    let exe = std::env::current_exe().context("Failed to determine current executable path")?;
    let exe_str = exe.to_string_lossy().to_string();
    Ok((exe, exe_str))
}

pub(crate) fn warn_if_build_dir(exe_str: &str) {
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
pub(crate) fn xml_escape(s: &str) -> String {
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
pub(crate) fn shell_escape(s: &str) -> String {
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
}

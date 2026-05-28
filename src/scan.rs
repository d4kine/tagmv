use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "m4a", "flac", "ogg", "wma", "aac", "wav"];

/// Extract a file's extension, falling back to the substring after the last
/// dot in the file name when [`Path::extension`] returns nothing.
pub fn file_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(OsStr::to_str).or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.rsplit_once('.').map(|(_, ext)| ext))
    })
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

pub fn scan_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn is_audio_file_matches_known_extensions() {
        assert!(is_audio_file(Path::new("song.mp3")));
        assert!(is_audio_file(Path::new("song.FLAC")));
        assert!(is_audio_file(Path::new("/a/b/track.M4a")));
        assert!(!is_audio_file(Path::new("cover.jpg")));
        assert!(!is_audio_file(Path::new("README")));
    }

    #[test]
    fn is_hidden_detects_dotfiles() {
        assert!(is_hidden(".hidden.mp3"));
        assert!(!is_hidden("visible.mp3"));
    }

    #[test]
    fn file_extension_handles_fallback() {
        assert_eq!(file_extension(Path::new("song.mp3")), Some("mp3"));
        assert_eq!(file_extension(Path::new("/a/b/track.flac")), Some("flac"));
        assert_eq!(file_extension(Path::new("noext")), None);
    }

    #[test]
    fn scan_files_non_recursive_filters_correctly() {
        let tmp = std::env::temp_dir().join("tagmv_scan_nonrec");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        fs::write(tmp.join("a.mp3"), "x").unwrap();
        fs::write(tmp.join("b.flac"), "x").unwrap();
        fs::write(tmp.join("cover.jpg"), "x").unwrap();
        fs::write(tmp.join(".hidden.mp3"), "x").unwrap();
        fs::create_dir_all(tmp.join("sub")).unwrap();
        fs::write(tmp.join("sub/c.mp3"), "x").unwrap();

        let found = scan_files(&tmp, false).unwrap();
        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert_eq!(names, vec!["a.mp3", "b.flac"]);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn scan_files_recursive_skips_unsorted_and_hidden() {
        let tmp = std::env::temp_dir().join("tagmv_scan_rec");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        fs::write(tmp.join("a.mp3"), "x").unwrap();
        fs::create_dir_all(tmp.join("sub")).unwrap();
        fs::write(tmp.join("sub/b.mp3"), "x").unwrap();
        fs::create_dir_all(tmp.join("_Unsorted")).unwrap();
        fs::write(tmp.join("_Unsorted/c.mp3"), "x").unwrap();
        fs::create_dir_all(tmp.join(".hiddendir")).unwrap();
        fs::write(tmp.join(".hiddendir/d.mp3"), "x").unwrap();

        let found = scan_files(&tmp, true).unwrap();
        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert_eq!(names, vec!["a.mp3", "b.mp3"]);

        let _ = fs::remove_dir_all(&tmp);
    }
}

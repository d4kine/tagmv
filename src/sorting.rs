use crate::tags::TrackMetadata;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Windows/FAT32 reserved device names that are invalid as filenames.
const RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

const MAX_CONFLICT_ATTEMPTS: u32 = 10_000;

/// Sanitize a string for safe use in filenames.
/// Mirrors `slugify_for_filename` from rename_audio_by_tags.py.
pub fn sanitize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '/' | '\\' => out.push('-'),
            ':' | '*' | '?' | '"' | '<' | '>' | '|' => {}
            c if c.is_control() => {}
            _ => out.push(c),
        }
    }

    // Collapse whitespace
    let collapsed: String = out.split_whitespace().collect::<Vec<_>>().join(" ");

    // Trim dots and spaces
    let trimmed = collapsed
        .trim_matches(|c: char| c == '.' || c == ' ')
        .to_string();

    if trimmed.is_empty() {
        return "Unknown".to_string();
    }

    // Guard against reserved device names (for FAT32/exFAT compatibility)
    if RESERVED_NAMES
        .iter()
        .any(|r| r.eq_ignore_ascii_case(&trimmed))
    {
        return format!("_{}", trimmed);
    }

    trimmed
}

/// A planned file move operation.
#[derive(Debug)]
pub struct PlannedMove {
    pub source: PathBuf,
    pub dest: PathBuf,
    pub folder_name: String,
    pub file_name: String,
}

/// Compute destination path for a file with known tags.
pub fn compute_destination(base_dir: &Path, source: &Path, meta: &TrackMetadata) -> PlannedMove {
    let artist = sanitize(&meta.artist);
    let album = sanitize(&meta.album);
    let folder_name = format!("{} - {}", artist, album);

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_else(|| {
            source
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.rsplit_once('.').map_or("", |(_, e)| e))
                .unwrap_or("")
        });

    let title = meta
        .title
        .as_deref()
        .map(sanitize)
        .unwrap_or_else(|| {
            source
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

    let file_name = match meta.track_number {
        Some(n) => format!("{:02} - {}.{}", n, title, ext),
        None => format!("{}.{}", title, ext),
    };

    let dest = base_dir.join(&folder_name).join(&file_name);

    PlannedMove {
        source: source.to_path_buf(),
        dest,
        folder_name,
        file_name,
    }
}

/// Compute destination for unsorted files.
pub fn compute_unsorted_destination(base_dir: &Path, source: &Path) -> PlannedMove {
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let folder_name = "_Unsorted".to_string();
    let dest = base_dir.join(&folder_name).join(&file_name);

    PlannedMove {
        source: source.to_path_buf(),
        dest,
        folder_name,
        file_name,
    }
}

/// Resolve conflicts: both on-disk and intra-batch duplicates.
pub fn resolve_conflicts(moves: &mut [PlannedMove]) {
    let mut claimed: HashSet<PathBuf> = HashSet::new();

    for m in moves.iter_mut() {
        if m.source == m.dest {
            continue;
        }

        let mut candidate = m.dest.clone();
        let mut counter = 1u32;

        while candidate.exists() || claimed.contains(&candidate) {
            let stem = m
                .dest
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = m
                .dest
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let parent = m.dest.parent().unwrap();

            let new_name = if ext.is_empty() {
                format!("{} ({})", stem, counter)
            } else {
                format!("{} ({}).{}", stem, counter, ext)
            };

            candidate = parent.join(&new_name);

            counter = match counter.checked_add(1) {
                Some(n) if n <= MAX_CONFLICT_ATTEMPTS => n,
                _ => {
                    // Give up -- keep the last candidate and let execute_move
                    // report an error if it actually collides at runtime
                    break;
                }
            };
        }

        if candidate != m.dest {
            m.file_name = candidate
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            m.dest = candidate.clone();
        }

        claimed.insert(candidate);
    }
}

/// Execute a file move. Creates directories as needed.
/// Checks for conflicts at move time, uses rename first, falls back to
/// copy+delete only for cross-device moves (verifying file size).
pub fn execute_move(planned: &PlannedMove) -> Result<()> {
    if planned.source == planned.dest {
        return Ok(());
    }

    if let Some(parent) = planned.dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Re-check at move time: if destination appeared since planning, bail
    if planned.dest.exists() {
        bail!(
            "Destination already exists (appeared after planning): {}",
            planned.dest.display()
        );
    }

    match fs::rename(&planned.source, &planned.dest) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Only fall back to copy+delete for cross-device errors
            let dominated_by_cross_device = matches!(
                e.raw_os_error(),
                Some(libc::EXDEV)
            );
            if !dominated_by_cross_device {
                return Err(e).with_context(|| {
                    format!(
                        "Failed to move {} -> {}",
                        planned.source.display(),
                        planned.dest.display()
                    )
                });
            }

            let source_len = fs::metadata(&planned.source)
                .with_context(|| {
                    format!("Failed to read source metadata: {}", planned.source.display())
                })?
                .len();

            let bytes_copied = fs::copy(&planned.source, &planned.dest).with_context(|| {
                format!(
                    "Failed to copy {} -> {}",
                    planned.source.display(),
                    planned.dest.display()
                )
            })?;

            if bytes_copied != source_len {
                // Remove incomplete copy, keep source intact
                let _ = fs::remove_file(&planned.dest);
                bail!(
                    "Copy verification failed for {}: expected {} bytes, copied {}",
                    planned.source.display(),
                    source_len,
                    bytes_copied
                );
            }

            fs::remove_file(&planned.source).with_context(|| {
                format!("Failed to remove source: {}", planned.source.display())
            })?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sanitize_basic() {
        assert_eq!(sanitize("Hello World"), "Hello World");
        assert_eq!(sanitize("AC/DC"), "AC-DC");
        assert_eq!(sanitize("Back\\Slash"), "Back-Slash");
    }

    #[test]
    fn sanitize_removes_forbidden_chars() {
        assert_eq!(sanitize("What: is *this?"), "What is this");
        assert_eq!(sanitize("a\"b<c>d|e"), "abcde");
    }

    #[test]
    fn sanitize_control_chars() {
        assert_eq!(sanitize("hello\x00world\x1f"), "helloworld");
    }

    #[test]
    fn sanitize_whitespace_collapse() {
        assert_eq!(sanitize("  too   many   spaces  "), "too many spaces");
        // Tab is a control character, removed before whitespace collapse
        assert_eq!(sanitize("tabs\there"), "tabshere");
    }

    #[test]
    fn sanitize_dots_trimmed() {
        assert_eq!(sanitize("...leading"), "leading");
        assert_eq!(sanitize("trailing..."), "trailing");
        assert_eq!(sanitize("..both.."), "both");
    }

    #[test]
    fn sanitize_empty_becomes_unknown() {
        assert_eq!(sanitize(""), "Unknown");
        assert_eq!(sanitize("***"), "Unknown");
        assert_eq!(sanitize("..."), "Unknown");
        assert_eq!(sanitize("   "), "Unknown");
    }

    #[test]
    fn sanitize_reserved_names() {
        assert_eq!(sanitize("CON"), "_CON");
        assert_eq!(sanitize("con"), "_con");
        assert_eq!(sanitize("NUL"), "_NUL");
        assert_eq!(sanitize("PRN"), "_PRN");
        assert_eq!(sanitize("COM1"), "_COM1");
        assert_eq!(sanitize("LPT9"), "_LPT9");
        // Not reserved
        assert_eq!(sanitize("CONNECT"), "CONNECT");
        assert_eq!(sanitize("CONSOLE"), "CONSOLE");
    }

    #[test]
    fn sanitize_unicode_preserved() {
        assert_eq!(sanitize("Chlär"), "Chlär");
        assert_eq!(sanitize("Nørbak"), "Nørbak");
    }

    #[test]
    fn compute_destination_full_tags() {
        let base = PathBuf::from("/music");
        let source = PathBuf::from("/downloads/01 Song.m4a");
        let meta = TrackMetadata {
            artist: "Artist".to_string(),
            album: "Album".to_string(),
            title: Some("Song Title".to_string()),
            track_number: Some(1),
        };
        let result = compute_destination(&base, &source, &meta);
        assert_eq!(result.folder_name, "Artist - Album");
        assert_eq!(result.file_name, "01 - Song Title.m4a");
        assert_eq!(result.dest, PathBuf::from("/music/Artist - Album/01 - Song Title.m4a"));
    }

    #[test]
    fn compute_destination_no_track_number() {
        let base = PathBuf::from("/music");
        let source = PathBuf::from("/downloads/song.mp3");
        let meta = TrackMetadata {
            artist: "A".to_string(),
            album: "B".to_string(),
            title: Some("Title".to_string()),
            track_number: None,
        };
        let result = compute_destination(&base, &source, &meta);
        assert_eq!(result.file_name, "Title.mp3");
    }

    #[test]
    fn compute_destination_no_title_uses_stem() {
        let base = PathBuf::from("/music");
        let source = PathBuf::from("/downloads/03 Original Name.flac");
        let meta = TrackMetadata {
            artist: "X".to_string(),
            album: "Y".to_string(),
            title: None,
            track_number: Some(3),
        };
        let result = compute_destination(&base, &source, &meta);
        assert_eq!(result.file_name, "03 - 03 Original Name.flac");
    }

    #[test]
    fn compute_destination_special_chars_in_artist() {
        let base = PathBuf::from("/music");
        let source = PathBuf::from("/downloads/song.m4a");
        let meta = TrackMetadata {
            artist: "AC/DC".to_string(),
            album: "Back in Black".to_string(),
            title: Some("Hells Bells".to_string()),
            track_number: Some(1),
        };
        let result = compute_destination(&base, &source, &meta);
        assert_eq!(result.folder_name, "AC-DC - Back in Black");
    }

    #[test]
    fn compute_unsorted_preserves_filename() {
        let base = PathBuf::from("/music");
        let source = PathBuf::from("/downloads/weird file.m4a");
        let result = compute_unsorted_destination(&base, &source);
        assert_eq!(result.folder_name, "_Unsorted");
        assert_eq!(result.file_name, "weird file.m4a");
        assert_eq!(result.dest, PathBuf::from("/music/_Unsorted/weird file.m4a"));
    }

    #[test]
    fn resolve_conflicts_intra_batch() {
        let mut moves = vec![
            PlannedMove {
                source: PathBuf::from("/a/file1.mp3"),
                dest: PathBuf::from("/b/song.mp3"),
                folder_name: "folder".to_string(),
                file_name: "song.mp3".to_string(),
            },
            PlannedMove {
                source: PathBuf::from("/a/file2.mp3"),
                dest: PathBuf::from("/b/song.mp3"),
                folder_name: "folder".to_string(),
                file_name: "song.mp3".to_string(),
            },
        ];
        resolve_conflicts(&mut moves);
        assert_eq!(moves[0].file_name, "song.mp3");
        assert_eq!(moves[1].file_name, "song (1).mp3");
        assert_ne!(moves[0].dest, moves[1].dest);
    }

    #[test]
    fn resolve_conflicts_already_in_place_skipped() {
        let same = PathBuf::from("/music/Artist - Album/01 - Song.m4a");
        let mut moves = vec![PlannedMove {
            source: same.clone(),
            dest: same.clone(),
            folder_name: "Artist - Album".to_string(),
            file_name: "01 - Song.m4a".to_string(),
        }];
        resolve_conflicts(&mut moves);
        assert_eq!(moves[0].dest, same);
    }

    #[test]
    fn execute_move_creates_dirs_and_moves() {
        let tmp = std::env::temp_dir().join("tagmv_test_move");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let source = tmp.join("source.txt");
        fs::write(&source, "test content").unwrap();

        let dest = tmp.join("subdir/dest.txt");
        let planned = PlannedMove {
            source: source.clone(),
            dest: dest.clone(),
            folder_name: "subdir".to_string(),
            file_name: "dest.txt".to_string(),
        };

        execute_move(&planned).unwrap();
        assert!(!source.exists());
        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "test content");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn execute_move_skips_same_path() {
        let same = PathBuf::from("/nonexistent/same.mp3");
        let planned = PlannedMove {
            source: same.clone(),
            dest: same,
            folder_name: "f".to_string(),
            file_name: "same.mp3".to_string(),
        };
        // Should not error even though path doesn't exist
        execute_move(&planned).unwrap();
    }

    #[test]
    fn execute_move_refuses_existing_dest() {
        let tmp = std::env::temp_dir().join("tagmv_test_conflict");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let source = tmp.join("a.txt");
        let dest = tmp.join("b.txt");
        fs::write(&source, "a").unwrap();
        fs::write(&dest, "b").unwrap();

        let planned = PlannedMove {
            source,
            dest: dest.clone(),
            folder_name: "f".to_string(),
            file_name: "b.txt".to_string(),
        };

        let result = execute_move(&planned);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already exists"));

        let _ = fs::remove_dir_all(&tmp);
    }
}

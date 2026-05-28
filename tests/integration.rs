use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use tagmv::plan_moves;
use tagmv::sorting::execute_move;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a fresh, unique temp directory for a test.
fn fresh_dir(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tagmv_it_{}_{}_{}", tag, std::process::id(), n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn untagged_files_route_to_unsorted_and_move() {
    let dir = fresh_dir("unsorted");
    fs::write(dir.join("a.mp3"), "aaa").unwrap();
    fs::write(dir.join("b.flac"), "bbb").unwrap();

    let moves = plan_moves(&dir, false).unwrap();
    assert_eq!(moves.len(), 2);
    assert!(moves.iter().all(|m| m.folder_name == "_Unsorted"));

    // Planning alone must not move anything.
    assert!(dir.join("a.mp3").exists());
    assert!(dir.join("b.flac").exists());

    for m in &moves {
        execute_move(m).unwrap();
    }

    assert!(!dir.join("a.mp3").exists());
    assert!(!dir.join("b.flac").exists());
    assert!(dir.join("_Unsorted/a.mp3").exists());
    assert!(dir.join("_Unsorted/b.flac").exists());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn conflict_with_existing_file_is_renumbered() {
    let dir = fresh_dir("conflict");
    // A file already sitting at the destination path.
    fs::create_dir_all(dir.join("_Unsorted")).unwrap();
    fs::write(dir.join("_Unsorted/d.mp3"), "existing").unwrap();
    // The source file that will collide on move.
    fs::write(dir.join("d.mp3"), "incoming").unwrap();

    let moves = plan_moves(&dir, false).unwrap();
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].file_name, "d (1).mp3");

    execute_move(&moves[0]).unwrap();
    assert!(dir.join("_Unsorted/d (1).mp3").exists());
    assert_eq!(
        fs::read_to_string(dir.join("_Unsorted/d.mp3")).unwrap(),
        "existing"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn non_audio_files_are_ignored() {
    let dir = fresh_dir("nonaudio");
    fs::write(dir.join("cover.jpg"), "img").unwrap();
    fs::write(dir.join("notes.txt"), "txt").unwrap();

    let moves = plan_moves(&dir, false).unwrap();
    assert!(moves.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

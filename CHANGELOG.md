# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- `tagmv status` subcommand: reports whether the context menu is installed; on macOS also whether it is enabled in System Settings (read from `pbs.plist`). `make install` runs it, `make menu-status` exposes it
- macOS Quick Action reports results: last summary line as a notification, failures as a dialog with the full output; selected files map to their parent folder, each folder is processed once; a dialog is shown when the binary is missing
- `contrib/tagmv.workflow`: ready-made Quick Action bundle (double-click to install) that resolves `tagmv` from `PATH` at run time; a snapshot test keeps it in sync with the generator
- `tagmv install` on macOS refreshes the Services cache (`pbs -update`) so the Quick Action appears without restarting Finder
- Album artist tag is preferred over the track artist for the folder name (compilations stay together)
- Scanning a folder that is already named `Artist - Album` (or `_Unsorted`) renames files in place instead of nesting a second folder level
- Approval prompt can be aborted with Esc/`q` ("Aborted, nothing moved")

### Changed
- Exit code is 1 when at least one move failed
- Track number `0` is treated as missing (no `00 - ` prefix)
- Sanitized path components are capped at 100 bytes (avoids `ENAMETOOLONG`)
- Symlinks are skipped during scanning
- `same-file` dependency (already pulled in by `walkdir`) for identity checks

### Fixed
- Re-running with `-r` no longer renames `Title (1).ext` to `(2)`, `(3)`, ... on every run; a destination that is the same file as the source counts as already in place (also fixes spurious `(1)` suffixes for case-only differences on APFS/NTFS)
- Ctrl-C in the approval prompt no longer leaves the terminal cursor hidden

## [0.3.0] - 2026-05-29

### Added
- Interactive approval prompt (`src/approve.rs`): after the preview, a `dialoguer` MultiSelect shows all planned moves pre-checked; deselect individual items with Space, confirm with Enter
- `--yes` / `-y` flag: skip the prompt and execute all moves without interaction
- `dialoguer 0.11` dependency

### Changed
- Prompt is auto-skipped when stdin is not a TTY (piped input, CI, file manager context menu)
- `execute_all()` now receives only pending moves (`&[&PlannedMove]`), excluding already-in-place files


## [0.2.0] - 2026-05-28

**MINOR** - default behaviour changed (`--execute` removed, execute-by-default introduced). Breaking for scripts that relied on `--execute`.

### Added
- `Makefile` with targets: `build`, `release`, `test`, `fmt`, `fmt-check`, `clippy`, `install`, `menu-install`, `menu-uninstall`, `clean`, `help`
- Integration test suite (`tests/integration.rs`): unsorted fallback, conflict renumbering, non-audio filtering
- Unit tests inside `scan.rs` and `install/mod.rs`
- "Already in place" detection: files already at their correct destination are shown in grey and counted as skipped
- Execution summary: reports moved, unsorted, skipped, and folder counts
- `rustfmt.toml` and `#![warn(clippy::all)]` at crate root

### Changed
- `--execute` flag **removed**; execute is now the default. Use `--dry-run` / `-n` for preview-only mode
- Library split from binary: `main.rs` is now a thin shim; all logic lives in `lib.rs` and sub-modules (`cli.rs`, `scan.rs`, `display.rs`, `install/`)
- `install/` refactored into platform sub-modules: `mod.rs`, `macos.rs`, `linux.rs`, `windows.rs`
- `RunOptions` struct introduced as the public API surface for `run()` and `plan_moves()`


## [0.1.0] - 2026-02-16

**Initial release.**

### Added
- Reads audio tags (artist, album, title, track number) via `lofty 0.22`
- Organises files into `Artist - Album/NN - Title.ext` layout
- Files missing artist or album go to `_Unsorted/`
- `--recursive` / `-r` flag for scanning subdirectories
- Conflict resolution: appends `(1)`, `(2)`, … suffixes (up to 10 000 attempts)
- Cross-device move fallback: copy + verify + delete on `EXDEV`
- Filename sanitisation: removes FAT32-reserved characters, strips control chars, collapses whitespace, handles reserved names
- `install` / `uninstall` subcommands for file manager integration:
  - **macOS**: Finder Quick Action via Automator workflow
  - **Linux**: Nautilus scripts, Nemo action file, Dolphin service menu
  - **Windows**: Explorer context menu via `HKCU` registry key (no admin required)


[Unreleased]: https://github.com/d4kine/tagmv
[0.3.0]: https://github.com/d4kine/tagmv/tree/v0.3.0
[0.2.0]: https://github.com/d4kine/tagmv/tree/v0.2.0
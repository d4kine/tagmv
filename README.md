# tagmv

Rust CLI tool that organizes music files into `Artist - Album/01 - Title.ext` folder structure by reading audio tags. Moves files by default; use `--dry-run` to preview first.

## Usage

```
tagmv [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to sort (defaults to current directory)

Options:
  -n, --dry-run   Preview changes without moving any files
  -y, --yes       Skip the approval prompt and move all planned files
  -r, --recursive Scan subdirectories
  -h, --help      Print help
  -V, --version   Print version

Subcommands:
  install         Install file manager context menu
  uninstall       Remove file manager context menu
  status          Show whether the context menu is installed (and enabled on macOS)
```

### Dry-run preview

```
$ tagmv -n "/Users/chris/Downloads/Telegram Desktop"

tagmv v0.2.0 -- DRY RUN (no files will be moved)

Scanning: /Users/chris/Downloads/Telegram Desktop
Found 39 audio files

  Chlär - Breakthrough - EP/
    01 - Close Contact.m4a          <- 01 Close Contact.m4a
    02 - Pressure Point.m4a         <- 02 Pressure Point.m4a
    ...

Summary: 39 files -> 7 folders, 0 unsorted
```

### Move files

Running without `-n` prints the same preview and then, in an interactive
terminal, shows a checkbox list where you approve or deselect individual moves
before anything happens:

```
$ tagmv "/path/to/music"

? Confirm moves (space toggles, a selects all, enter confirms)
  [x] Chlär - Breakthrough - EP/01 - Close Contact.m4a  <- 01 Close Contact.m4a
  [x] Chlär - Breakthrough - EP/02 - Pressure Point.m4a <- 02 Pressure Point.m4a
  [ ] _Unsorted/random.mp3  <- random.mp3
```

Only the checked items are moved. Use `-y`/`--yes` to skip the prompt and move
everything. When there's no interactive terminal (piped input, CI, or the file
manager context menu) the prompt is skipped automatically and the full plan
runs.

### Context menu integration

```
tagmv install       # add "tagmv" to your file manager
tagmv uninstall     # remove it
tagmv status        # installed? enabled in System Settings (macOS)?
```

`make install` copies the binary and runs `tagmv status` afterwards, so a
missing or disabled context menu is reported right away.

`tagmv install` auto-detects the OS and installs the appropriate integration:

**macOS** -- Finder Quick Action (Automator workflow)

`tagmv install` writes `~/Library/Services/tagmv.workflow`, refreshes the
Services cache (`pbs -update`) and prints the status. `tagmv status` reads the
enabled state from `~/Library/Preferences/pbs.plist`, the same data behind
**System Settings -> Privacy & Security -> Extensions -> Finder**. If it
reports DISABLED, toggle **tagmv** on there. If the action does not show up in
Finder, run `killall Finder`.

Right-click a folder, or files inside it -> **Quick Actions** -> **tagmv**

The Quick Action runs `tagmv -y` on every selected folder (selected files map
to their parent folder, each folder once). The last summary line is shown as a
macOS notification; if any move fails, a dialog shows the full tagmv output.

Without running `tagmv install`, double-click
`contrib/tagmv.workflow` and confirm the install prompt. This
copy looks up `tagmv` on `PATH` at run time (fallback `~/bin/tagmv`) instead
of embedding an absolute binary path. It is generated from the same template;
`UPDATE_WORKFLOW=1 cargo test contrib_workflow` regenerates it.

**Linux** -- Nautilus, Nemo, and Dolphin

Installs context menu entries for all three file managers:
- Nautilus (GNOME): `~/.local/share/nautilus/scripts/tagmv`
- Nemo (Cinnamon): `~/.local/share/nemo/actions/tagmv.nemo_action`
- Dolphin (KDE): `~/.local/share/kio/servicemenus/tagmv.desktop`

Right-click a folder -> **Scripts** or **Actions** -> **tagmv**

**Windows** -- Explorer context menu (registry)

Adds entries under `HKCU\Software\Classes\Directory\shell\tagmv` (no admin needed).
Right-click a folder in Explorer -> **tagmv**

> **Note:** The context menu moves files immediately -- there is no dry-run
> preview. Run `tagmv -n <path>` from the terminal first to preview changes.
> Exit code is 1 when at least one move failed, 0 otherwise (including
> an aborted approval prompt via Esc/q).

## Sorting rules

- Files with non-empty **artist** and **album** tags -> `Artist - Album/01 - Title.ext`
- **Album artist** is preferred over the track artist when set (keeps compilations in one folder)
- Files missing or with empty artist/album tags -> `_Unsorted/`
- Track numbers are zero-padded (`01`, `02`, ...); files without a track number (or `0`) omit the prefix
- If no title tag, the original filename stem is used
- Files already at their correct destination are skipped; a destination that
  is the same file as the source (case-only difference, or an earlier `(1)`
  suffix) also counts as in place and is never renamed again
- If the scanned folder is already named `Artist - Album` (or `_Unsorted`),
  files are renamed inside it instead of being nested one level deeper
- Conflict resolution appends `(1)`, `(2)`, etc.
- Cross-device moves fall back to copy + delete

## Scanning behavior

- By default only the top-level directory is scanned; use `-r` for subdirectories
- Hidden files and directories (dotfiles) and symlinks are always skipped
- The `_Unsorted/` directory is skipped during recursive scanning

## Supported formats

mp3, m4a, flac, ogg, wma, aac, wav

Tag reading is handled by [lofty](https://crates.io/crates/lofty).

## Filename sanitization

- `/` and `\` -> `-` (handles artists like AC/DC)
- Removes `: * ? " < > |` and control characters
- Collapses whitespace, trims dots and spaces
- Each component (artist, album, title) is capped at 100 bytes on a character boundary
- Empty result after sanitization falls back to "Unknown"

## Build

```
cargo build --release
cp target/release/tagmv ~/bin/
```

Ensure `~/bin` exists and is on your `PATH`.

## License

[MIT](LICENSE)

## Support

If you find this useful, consider supporting the project:

<p>
  <a href="https://buymeacoffee.com/d4kine"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee"></a>
  <a href="https://ko-fi.com/d4kine"><img src="https://img.shields.io/badge/Ko--fi-FF5E5B?style=flat&logo=ko-fi&logoColor=white" alt="Ko-fi"></a>
  <a href="https://paypal.me/kaubisch"><img src="https://img.shields.io/badge/PayPal-00457C?style=flat&logo=paypal&logoColor=white" alt="PayPal"></a>
  <!--
  <a href="https://patreon.com/YOUR_USERNAME"><img src="https://img.shields.io/badge/Patreon-F96854?style=flat&logo=patreon&logoColor=white" alt="Patreon"></a>
  -->
</p>

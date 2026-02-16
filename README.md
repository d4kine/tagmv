# tagmv

Rust CLI tool that organizes music files into `Artist - Album/01 - Title.ext` folder structure by reading audio tags. Dry-run by default.

## Usage

```
tagmv [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to sort (defaults to current directory)

Options:
  --execute       Actually move files (default is dry-run preview)
  -r, --recursive Scan subdirectories
  -h, --help      Print help
  -V, --version   Print version

Subcommands:
  install         Install file manager context menu
  uninstall       Remove file manager context menu
```

### Dry-run preview

```
$ tagmv "/Users/chris/Downloads/Telegram Desktop"

tagmv v0.1.0 -- DRY RUN (use --execute to move files)

Scanning: /Users/chris/Downloads/Telegram Desktop
Found 39 audio files

  Chlär - Breakthrough - EP/
    01 - Close Contact.m4a          <- 01 Close Contact.m4a
    02 - Pressure Point.m4a         <- 02 Pressure Point.m4a
    ...

Summary: 39 files -> 7 folders, 0 unsorted
```

### Execute moves

```
$ tagmv --execute "/path/to/music"
```

### Context menu integration

```
tagmv install       # add "Sort Music by Tags" to your file manager
tagmv uninstall     # remove it
```

`tagmv install` auto-detects the OS and installs the appropriate integration:

**macOS** -- Finder Quick Action (Automator workflow)

After running `tagmv install`:
1. Open **System Settings -> Privacy & Security -> Extensions -> Finder**
2. Enable **Sort Music by Tags**
3. If it doesn't appear, run `killall Finder`
4. Right-click a folder -> **Quick Actions** -> **Sort Music by Tags**

**Linux** -- Nautilus, Nemo, and Dolphin

Installs context menu entries for all three file managers:
- Nautilus (GNOME): `~/.local/share/nautilus/scripts/Sort Music by Tags`
- Nemo (Cinnamon): `~/.local/share/nemo/actions/tagmv.nemo_action`
- Dolphin (KDE): `~/.local/share/kio/servicemenus/tagmv.desktop`

Right-click a folder -> **Scripts** or **Actions** -> **Sort Music by Tags**

**Windows** -- Explorer context menu (registry)

Adds entries under `HKCU\Software\Classes\Directory\shell\tagmv` (no admin needed).
Right-click a folder in Explorer -> **Sort Music by Tags**

> **Note:** The context menu runs in execute mode (`--execute`) immediately --
> there is no dry-run preview. Run `tagmv <path>` from the terminal first
> to preview changes.

## Sorting rules

- Files with non-empty **artist** and **album** tags -> `Artist - Album/01 - Title.ext`
- Files missing or with empty artist/album tags -> `_Unsorted/`
- Track numbers are zero-padded (`01`, `02`, ...); files without a track number omit the prefix
- If no title tag, the original filename stem is used
- Files already at their correct destination are skipped
- Conflict resolution appends `(1)`, `(2)`, etc.
- Cross-device moves fall back to copy + delete

## Scanning behavior

- By default only the top-level directory is scanned; use `-r` for subdirectories
- Hidden files and directories (dotfiles) are always skipped
- The `_Unsorted/` directory is skipped during recursive scanning

## Supported formats

mp3, m4a, flac, ogg, wma, aac, wav

Tag reading is handled by [lofty](https://crates.io/crates/lofty).

## Filename sanitization

- `/` and `\` -> `-` (handles artists like AC/DC)
- Removes `: * ? " < > |` and control characters
- Collapses whitespace, trims dots and spaces
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
  <a href="https://buymeacoffee.com/YOUR_USERNAME"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee"></a>
  <a href="https://ko-fi.com/YOUR_USERNAME"><img src="https://img.shields.io/badge/Ko--fi-FF5E5B?style=flat&logo=ko-fi&logoColor=white" alt="Ko-fi"></a>
  <a href="https://patreon.com/YOUR_USERNAME"><img src="https://img.shields.io/badge/Patreon-F96854?style=flat&logo=patreon&logoColor=white" alt="Patreon"></a>
  <a href="https://paypal.me/YOUR_USERNAME"><img src="https://img.shields.io/badge/PayPal-00457C?style=flat&logo=paypal&logoColor=white" alt="PayPal"></a>
</p>

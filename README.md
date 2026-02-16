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
  install         Install macOS Finder Quick Action
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

### Finder Quick Action setup

1. Build and install the binary:

   ```
   cargo build --release
   cp target/release/tagmv ~/bin/
   ```

2. Install the Quick Action workflow:

   ```
   ~/bin/tagmv install
   ```

3. Enable the extension in macOS:

   - Open **System Settings -> Privacy & Security -> Extensions -> Finder**
     (on older macOS: **System Preferences -> Extensions -> Finder Extensions**)
   - Check **Sort Music by Tags**

4. If the action doesn't appear, refresh Finder:

   ```
   killall Finder
   ```

5. Use it: right-click a folder in Finder -> **Quick Actions** -> **Sort Music by Tags**

> **Note:** The Quick Action runs in execute mode (`--execute`) immediately --
> there is no dry-run preview. Run `tagmv <path>` from the terminal first
> to preview changes before using the Quick Action.

To uninstall the Quick Action:

```
rm -rf ~/Library/Services/Sort\ Music\ by\ Tags.workflow
```

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

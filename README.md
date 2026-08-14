# JA2 Stracciatella Savegame Analyzer

`ja2-savegame` is a standalone, read-only command-line tool that shows the
strategic locations of NPCs in Jagged Alliance 2 Stracciatella savegames. It
reads `.sav` files directly; JA2 does not need to be installed or running.

## Download

Download the latest binary from the
[GitHub releases page](https://github.com/manuelseeger/ja2-savegame-print/releases/latest):

- **Linux x86_64:** `ja2-savegame-linux-x86_64`
- **Windows x86_64:** `ja2-savegame-windows-x86_64.exe`
- **macOS x86_64:** `ja2-savegame-macos-x86_64`

On Linux or macOS, make the downloaded file executable and optionally rename it:

```sh
# macOS (use ja2-savegame-linux-x86_64 on Linux)
chmod +x ja2-savegame-macos-x86_64
mv ja2-savegame-macos-x86_64 ja2-savegame
```

## Usage

Inspect one savegame:

### Linux

```sh
./ja2-savegame /path/to/savegame.sav
```

### macOS

```sh
./ja2-savegame /path/to/savegame.sav
```

### Windows (PowerShell)

```powershell
.\ja2-savegame-windows-x86_64.exe C:\path\to\savegame.sav
```

By default, the tool lists characters from the original game whose location is
recorded in the save. Sectors use the familiar JA2 notation, such as `O4` or
`A9-2` for underground locations. The saved coordinates are also shown.

### Options

```sh
# Show only selected characters (repeatable; capitalization does not matter)
ja2-savegame file.sav --npc Hamous --npc Skyrider

# --include-npc is the longer form of --npc
ja2-savegame file.sav --include-npc Devin

# Exclude a character, even when also selected with --npc
ja2-savegame file.sav --exclude-npc Carmen

# Include every character, even those with no known name or location
ja2-savegame file.sav --all-profiles

# Output as JSON
ja2-savegame file.sav --json
ja2-savegame file.sav --json --pretty

# Show diagnostic details (-vv shows more)
ja2-savegame file.sav -v
ja2-savegame file.sav -vv
```

You can inspect one save file at a time. Character selection accepts the usual
name, full name, or nickname.

## Supported savegames

The tool supports normal saves from the **English edition** of:

- JA2 Stracciatella **v0.19.0 through v0.22.1**
- development builds after v0.22.1, up to the version listed under
  [Format reference](#format-reference)

It does not support:

- saves from Stracciatella v0.18.0 or older
- newer Stracciatella versions not listed above
- the old Linux-specific save format
- saves from the German edition
- JA2 1.13 saves
- saves whose format has been changed by a mod

Damaged and unsupported files are rejected with an error. A German save may
produce an error about character data or a checksum because the save itself
does not reliably identify the game edition.

A reported sector is the location recorded in the save; it does not always
mean that the character is currently visible on the tactical map. When the
save contains enough information, the output indicates whether a character is
placed, dead, unavailable, or recruited.

## Known limitations

- Stock NPC/RPC classification comes from Stracciatella's stock profile
  metadata. Unknown or modded profile IDs are not classified as stock NPCs;
  use `--all-profiles` to inspect them.
- Saved names and nicknames are used for display because the original stock
  display strings come from the licensed game data.
- Parsing intentionally stops after all merc profiles have been structurally
  decoded and checksum-validated. Later save sections are unnecessary for NPC
  location analysis.

## Format reference

The binary format and stock profile mapping are pinned to JA2 Stracciatella
commit:

```text
dcc20b3c24b3e49ccd16e9d4ae87dcd20b9e51ea
```

Run `ja2-savegame --source-version` to print this value.

Important constants were derived from `SaveLoadGame.cc`, `Laptop.cc`,
`LoadSaveMercProfile.cc`, `Tactical_Save.cc`, and
`assets/externalized/mercs-profile-info.json` at that revision. The bundled
save-obfuscation table and NPC/RPC mapping record their source-file SHA-256
hashes.

## Build from source

Install a stable Rust toolchain with [rustup](https://rustup.rs/), clone this
repository, and run:

```sh
cargo build --release --locked
```

The resulting executable is:

- Linux and macOS: `target/release/ja2-savegame`
- Windows: `target\release\ja2-savegame.exe`

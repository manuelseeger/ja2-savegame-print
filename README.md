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
./ja2-savegame inspect /path/to/savegame.sav
```

### macOS

```sh
./ja2-savegame inspect /path/to/savegame.sav
```

### Windows (PowerShell)

```powershell
.\ja2-savegame-windows-x86_64.exe inspect C:\path\to\savegame.sav
```

The default output contains stock NPCs and RPCs that have a meaningful saved
strategic location. Surface sectors use JA2 notation such as `O4`; underground
sectors include their depth, such as `A9-2`. Raw coordinates are shown as well.

### Options

```sh
# Show selected NPCs (repeatable, case-insensitive)
ja2-savegame inspect file.sav --npc Hamous --npc Skyrider

# --include-npc is the long form of --npc
ja2-savegame inspect file.sav --include-npc Devin

# Exclusions take precedence over inclusions
ja2-savegame inspect file.sav --exclude-npc Carmen

# Include all 170 profiles, including unmapped or unplaced profiles
ja2-savegame inspect file.sav --all-profiles

# Machine-readable output
ja2-savegame inspect file.sav --json
ja2-savegame inspect file.sav --json --pretty

# Show parsed section offsets on stderr
ja2-savegame inspect file.sav -v
ja2-savegame inspect file.sav -vv

# Print the pinned Stracciatella source revision
ja2-savegame --source-version
```

Each `inspect` invocation accepts exactly one save file. Name filters match the
stock canonical name, saved full name, and saved nickname.

## Supported savegames

The tool deliberately supports only normal portable **English-edition**
Stracciatella save format versions **102 and 103**.

It does not support:

- save format version 101 or older
- unknown future save versions
- legacy 688-byte Stracciatella Linux saves
- German-edition save encoding
- JA2 1.13 saves
- arbitrary mod-defined save layouts

Malformed and unsupported files are rejected with an error rather than parsed
heuristically. An unsupported German save may be reported as a profile
decoding or checksum failure because the header does not reliably identify the
edition.

A valid saved sector is strategic state; it does not necessarily prove that an
NPC is currently spawned in a loaded tactical map. Output distinguishes states
such as placed, not currently placed, dead, unavailable, and recruited where
the persisted profile fields permit it.

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

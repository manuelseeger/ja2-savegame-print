# JA2 Stracciatella Savegame Analyzer

`ja2-savegame` is a standalone, read-only Rust CLI for inspecting NPC strategic
locations in Jagged Alliance 2 Stracciatella `.sav` files. It parses the save
container and all 170 saved merc profiles directly; JA2 does not need to be
installed or launched at runtime.

## Build

Rust stable is required.

### Linux

```sh
cargo build --release
./target/release/ja2-savegame inspect file.sav
```

### Windows 10/11 (PowerShell)

Install Rust with [rustup](https://rustup.rs/), then run:

```powershell
cargo build --release
.\target\release\ja2-savegame.exe inspect C:\Games\JA2\SavedGames\file.sav
```

The implementation uses only portable Rust and accepts platform-native paths.

## Usage

```sh
ja2-savegame inspect file.sav
ja2-savegame inspect file.sav --json
ja2-savegame inspect file.sav --json --pretty
ja2-savegame inspect file.sav --npc Hamous --npc Skyrider
ja2-savegame inspect file.sav --include-npc Devin --exclude-npc Hamous
ja2-savegame inspect file.sav --all-profiles
ja2-savegame inspect file.sav -vv
ja2-savegame --source-version
```

Every invocation of `inspect` accepts exactly one save path. `--npc` is an
alias for repeatable `--include-npc`; matching is case-insensitive against the
stock canonical name and the saved full name and nickname. Exclusions take
precedence. By default, output contains source-classified stock NPC/RPC
profiles with a meaningful saved sector. `--all-profiles` exposes every parsed
profile, including unmapped and unplaced entries; name filters still apply.

Surface sectors use standard notation (`x=15,y=4,z=0` is `O4`). Underground
levels append depth (`A9-2`). JSON always retains raw `x`, `y`, and `z`; invalid
or sentinel coordinates have no `name` field and are never converted to a
valid sector. Profile output also reports whether the saved fields indicate
placed, not currently placed, dead, unavailable, recruited, or unknown state.
A saved valid sector is strategic state, not proof that an NPC is currently
spawned in a loaded tactical map.

Debug output from `-v`/`-vv` goes to stderr, so JSON on stdout remains valid.

## Supported format

The parser deliberately supports only normal portable **English-edition**
Stracciatella save format versions **102 and 103**. It rejects version 101 and
older, unknown future versions, malformed data, and recognizable legacy
688-byte Stracciatella Linux headers. German-edition encoding, German language
auto-detection, JA2 1.13, arbitrary mod-defined layouts, and historical save
formats are not supported. An unsupported German save can therefore appear as
a profile decoding/checksum failure.

The binary-format reference is pinned to JA2 Stracciatella commit (confirmed as
the intended upstream `master` review target during implementation):

```text
dcc20b3c24b3e49ccd16e9d4ae87dcd20b9e51ea
```

`--source-version` prints that pin. Important constants are traced in code to
`SaveLoadGame.cc`, `Laptop.cc`, `LoadSaveMercProfile.cc`, and
`Tactical_Save.cc`. The bundled 228 save-obfuscation rotation rows and stock
NPC/RPC table were derived from that exact source and record the source-file
SHA-256 hashes.

## Known limitations

* Stock NPC/RPC eligibility comes from pinned
  `assets/externalized/mercs-profile-info.json`. A modded or unknown profile ID
  is not silently classified as a stock NPC; use `--all-profiles` to see it.
* Saved names and nicknames remain the primary display values because stock
  display strings originate in licensed `prof.dat`, not the source metadata.
* Parsing intentionally stops after structurally decoding and checksum-checking
  all merc profiles. Later save sections are not needed for location analysis.
* No attempt is made to infer whether a strategically placed profile is
  tactically spawned beyond the status and placement fields persisted in
  `MERCPROFILESTRUCT`.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

The fixture integration tests parse each save independently and do not assert
hard-coded NPC locations.

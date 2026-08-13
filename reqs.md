# JA2 Stracciatella Savegame Analyzer

## 1. Objective

Build a standalone Rust command-line tool that analyzes Jagged Alliance 2 Stracciatella `.sav` files.

These files are JA2/Stracciatella saved-game files rather than event-stream formats.

The primary purpose is to inspect NPC strategic locations without launching JA2.

### Primary requirement

For the single supplied `.sav` file, output the current strategic locations of NPCs.

At minimum, support named NPCs whose locations are stored in `MERCPROFILESTRUCT`, including examples such as:

- Hamous
- Skyrider
- Devin
- Carmen
- Micky
- Gabby

Do not hard-code this list as the only supported NPCs. Output all stock profiles classified as NPC or RPC by the supported Stracciatella content for which a meaningful location exists. `--all-profiles` must still expose every parsed profile, including profiles whose NPC type cannot be determined.

### Supported scope

The initial implementation targets only the normal portable **English-language** save format used by the latest Stracciatella releases/source:

- save format version 102, represented by the supplied fixture and the latest release line
- save format version 103, used by current Stracciatella source at the pinned review commit

The authoritative source review is pinned to commit `dcc20b3c24b3e49ccd16e9d4ae87dcd20b9e51ea`. Reconfirm that this is still the intended latest source before implementation; if the target changes, update this specification and its supported-version list explicitly rather than silently widening compatibility.

This narrow two-version target does not imply general backward compatibility. Older save versions, the legacy Stracciatella Linux format, German-edition encoding, JA2 1.13, and arbitrary mod-defined save formats are explicitly unsupported. Unsupported inputs must fail clearly rather than being parsed heuristically.

# 2. Authoritative Reference Implementation

Use JA2 Stracciatella source code as the specification for the binary format.

Repository:

```text
https://github.com/ja2-stracciatella/ja2-stracciatella
```

Do not infer binary layouts from one supplied save if the corresponding Stracciatella serialization/deserialization code exists.

The implementation should inspect the repository source directly while being built. Use commit `dcc20b3c24b3e49ccd16e9d4ae87dcd20b9e51ea` as the pinned format specification unless the supported latest-source target is explicitly updated first. Record the resulting pin in the code and README, and expose it through `--source-version`.

Only the English edition's current save behavior is authoritative for this implementation. Do not add German-edition encryption-set calculation or language auto-detection.

Particularly important files/symbols are listed below.

---

# 3. Relevant Stracciatella Source

## Save container

Primary file:

```text
src/game/SaveLoadGame.cc
src/game/SaveLoadGame.h
```

Important symbols:

```text
SaveGame
LoadSavedGame
SaveHeader
ParseSavedGameHeader
ExtractSavedGameHeaderFromFile
SAVED_GAME_HEADER
CalcJA2EncryptionSet
SaveMercProfiles
LoadMercProfiles
SaveSoldierStructure
SaveNPCInfoToSaveGameFile
NewJA2EncryptedFileWrite
NewJA2EncryptedFileRead
```

`SaveGame()` is especially important because it defines the serialized sections and their order.

Treat its write order as the canonical top-level save layout.

---

# 4. Save Header

Current normal Windows/portable save header size:

```text
432 bytes
```

Stracciatella source defines:

```cpp
SAVED_GAME_HEADER::ON_DISK_SIZE = 432
```

An old Stracciatella Linux representation uses a 688-byte header, but it is outside the supported scope. The parser must recognize and reject that layout with an explicit unsupported-format error; it must not attempt to parse it as a normal header.

Use the current `ParseSavedGameHeader` field layout for the normal 432-byte header. Consult `ExtractSavedGameHeaderFromFile` only to understand how the legacy layout differs and how it can be rejected safely.

Important header fields include:

```text
uiSavedGameVersion
zGameVersionNumber
sSavedGameDesc
uiDay
ubHour
ubMin
sSector
ubNumOfMercsOnPlayersTeam
iCurrentBalance
uiCurrentScreen
fAlternateSector
fWorldLoaded
ubLoadScreenID
sInitialGameOptions
uiSaveStateSize
```
---

# 5. Top-Level Parsing Strategy

The parser must not search for byte signatures and assume fixed absolute offsets into the complete file.

Instead:

1. Parse the save header.
2. Obtain `uiSavedGameVersion`.
3. Verify that the save version is one of the explicitly supported current versions.
4. Follow the same section order and applicable version conditions used by `LoadSavedGame()`.
5. Implement readers/skippers for enough preceding sections to reach the merc-profile array.
6. Track the current byte offset.
7. Parse all merc profiles structurally.
8. Reject impossible sizes/counts instead of reading outside the file.

The implementation does not initially need semantic models for every save section.

For sections irrelevant to this tool, it is acceptable to implement:

```rust
fn skip_section(...)
```

provided the size can be derived safely from the actual serialized format.

Do not copy magic offsets from a sample `.sav`.

Parsing sections after the merc-profile array is not required for the initial implementation. Successful structural parsing and validation of every expected profile is the intended stopping point; reaching or validating the physical end of the save is not required.

---

# 6. Merc/NPC Profiles

NPC strategic locations are serialized in the game's merc profile array.

Relevant files:

```text
src/game/Tactical/LoadSaveMercProfile.cc
src/game/Tactical/LoadSaveMercProfile.h
src/game/Tactical/Soldier_Profile_Type.h
```

Relevant symbols:

```text
MERCPROFILESTRUCT
ExtractMercProfile
InjectMercProfile
```

`SaveMercProfiles()` iterates over the complete `gMercProfiles` array and serializes every profile.

In the normal save format, each serialized profile is:

```text
716 bytes
```

For the supported normal portable versions, verify the 716-byte constant against the pinned Stracciatella source and require it exactly. Historical and old-Linux profile layouts are unsupported and must not be implemented as compatibility fallbacks.

The serialized profile contains strategic location fields including:

```text
sSector.x
sSector.y
sSector.z
```

It also contains useful location-related fields such as:

```text
fUseProfileInsertionInfo
sGridNo
ubStrategicInsertionCode
usStrategicInsertionData
```

The first implementation only **must** expose strategic sector coordinates. Grid/insertion information is optional but desirable.

---

# 7. Save Encryption / Obfuscation

Merc profile blocks are not written as plain serialized structures.

`SaveMercProfiles()` effectively does:

```cpp
InjectMercProfile(...)
NewJA2EncryptedFileWrite(...)
```

Therefore the Rust parser must reproduce Stracciatella's save-data decoding.

Find and port the current implementation of:

```text
NewJA2EncryptedFileRead
NewJA2EncryptedFileWrite
CalcJA2EncryptionSet
```

and all constants/tables they depend on.

Requirements:

- implement it directly in Rust
- do not shell out to Stracciatella
- do not launch the game
- add unit tests for encrypt/decrypt symmetry where possible
- make encryption-set calculation depend on parsed save metadata exactly as English Stracciatella does
- hard-code the edition branch to English behavior; do not try the German calculation as a fallback
- fail profile validation clearly if data cannot be decoded using the supported English calculation

German-edition saves are not supported. The save header does not reliably identify that edition, so an unsupported German save may be reported as profile decoding/validation failure rather than edition-specific detection.

Do not describe this internally as strong cryptography. Treat it as the save encoding/obfuscation layer.

---

# 8. NPC Identification

A profile array index corresponds to a JA2 profile ID.

Derive a documented stock profile-ID/type/canonical-name table from the pinned Stracciatella source content rather than maintaining a manually invented table. Include both stock NPC and RPC profile types. The serialized profile's saved name and nickname remain the primary display values.

Search relevant source for profile IDs and profile data, including:

```text
MercProfile
Soldier_Profile
ContentManager
externalized profile data
NPC constants/enums
```

The parser should produce at least:

```rust
struct NpcLocation {
    profile_id: u16,
    name: String,
    nickname: Option<String>,
    sector: Option<Sector>,
}
```

Exact profile ID integer width should follow the source representation.

The save does not embed authoritative NPC/RPC type metadata for arbitrary mods. Therefore:

- default NPC eligibility is based on the bundled, source-derived stock NPC/RPC mapping
- `--all-profiles` exposes every parsed profile regardless of mapped type
- name filters match case-insensitively against both canonical stock names and saved names/nicknames
- unknown or modded profile IDs must not be silently classified as stock NPCs

`Sector` should conceptually be:

```rust
struct Sector {
    x: u8_or_u16,
    y: u8_or_u16,
    z: i8,
}
```

Use widths matching the serialized source data.

---

# 9. Sector Display

Output both raw coordinates and normal JA2 sector notation.

Example:

```text
x=15 y=4 z=0 -> O4
```

Map:

```text
x=1  -> A
x=2  -> B
...
x=16 -> P

y=1..16 -> 1..16
```

For underground sectors include depth explicitly, for example:

```text
A9-1
A9-2
```

or another documented unambiguous representation.

Always preserve raw `(x,y,z)` in JSON output even if a friendly display string is available.

---

# 10. Filtering NPCs

A profile may not currently have a meaningful strategic location. Treat `sSector` as the authoritative persisted strategic coordinate, but do not claim that a valid sector alone proves the NPC is currently spawned or tactically placeable.

Distinguish these cases where the fields in `MERCPROFILESTRUCT` support doing so:

```text
valid strategic sector
not currently placed
dead
unavailable
recruited
unknown/unsupported state
```

Do not silently convert invalid values to legitimate Arulco sectors.

The default human-readable output may hide obviously unused profiles, but JSON output should make filtering explicit.

Provide:

```text
--all-profiles
--npc NAME
--exclude-npc NAME
```

`--npc` (also available as `--include-npc`) and `--exclude-npc` must be repeatable and match NPC names case-insensitively. When one or more include filters are supplied, output only matching NPCs; exclusion filters are then applied, so exclusions take precedence when a name matches both. Without include filters, show all normally eligible NPCs except excluded names.

`--all-profiles` outputs every parsed profile, including profiles without valid strategic locations; name filters still apply.

---

# 11. CLI Interface

Binary name:

```text
ja2-savegame
```

Primary command:

```text
ja2-savegame inspect <FILE>
```

Each invocation must accept exactly one save file. Passing zero files or more than one file is a CLI usage error.

Example:

```text
ja2-savegame inspect savegame1.sav
```

Useful options:

```text
--json
--pretty
--all-profiles
--npc NAME
--include-npc NAME
--exclude-npc NAME
--source-version
```

Output must be exportable as either a plaintext list (the default when `--json` is absent) or valid JSON (`--json`). `--pretty` may format JSON for readability but must not make it invalid.

`--npc` is an alias for `--include-npc`. Include and exclude options should be repeatable:

```text
ja2-savegame inspect savegame1.sav \
  --include-npc Hamous \
  --include-npc Skyrider \
  --exclude-npc Devin
```

All NPC-name matching must be case-insensitive and consider canonical stock name, saved full name, and saved nickname. If an NPC matches both an include and an exclude filter, exclude it.

`--source-version` prints the pinned Stracciatella source commit used as the binary-format and stock-profile-mapping reference; it is distinct from the game-version string stored in each save header.

---

# 12. Default Human-Readable Output

Example format:

```text
savegame-001.sav

Save
  format version: 102
  game version:   Build 04.12.02
  time:           Day 2 13:46
  player sector:  O4
  world loaded:   false

NPCs
  Hamous      D9       (4,9,0)
  Skyrider    B15      (2,15,0)
  Devin       C6       (3,6,0)
  Carmen      C13      (3,13,0)
  Micky       H1       (8,1,0)

```

The displayed values above are examples only.

Never encode these locations as expected values.

---

# 13. JSON Output

JSON must be stable enough for scripts and valid JSON in both compact and `--pretty` modes. The plaintext-list mode must contain only human-readable text, not JSON fragments.

Suggested schema:

```json
{
  "file": "run-001.sav",
  "header": {
    "save_version": 102,
    "game_version": "Build 04.12.02",
    "description": "...",
    "day": 2,
    "hour": 13,
    "minute": 46,
    "sector": {
      "x": 15,
      "y": 4,
      "z": 0,
      "name": "O4"
    },
    "world_loaded": false
  },
  "npcs": [
    {
      "profile_id": 63,
      "name": "Example",
      "sector": {
        "x": 4,
        "y": 9,
        "z": 0,
        "name": "D9"
      }
    }
  ]
}
```

Again, numeric/profile values shown here are examples unless independently derived from Stracciatella source.

---

# 14. Version Compatibility

Do not build specifically for one sample file, but deliberately support only the normal portable English save versions emitted by the latest supported Stracciatella release/source. For pinned source commit `dcc20b3c24b3e49ccd16e9d4ae87dcd20b9e51ea`, the accepted set is versions 102 and 103.

Required behavior:

1. Read enough of the normal header to obtain `uiSavedGameVersion` safely.
2. Accept only the explicitly supported versions.
3. Follow the applicable gates from the pinned `LoadSavedGame()` implementation.
4. Recognize and reject the legacy 688-byte Stracciatella Linux layout.
5. Reject versions 101 and below rather than adding backward-compatibility paths.
6. Reject future unknown versions until their source format has been reviewed explicitly.
7. Fail clearly on malformed or unsupported input.

Example errors:

```text
unsupported JA2 save version 87; supported English portable versions: 102, 103
legacy Stracciatella Linux saves are not supported
```

Do not continue until a later `unexpected EOF` when the unsupported format can be identified at the header.

Keep version handling centralized even though its accepted set is intentionally narrow:

```rust
struct SaveContext {
    save_version: SupportedSaveVersion,
    encryption_set: ...,
}
```

Do not include old-Linux or German-edition switches in the supported parsing context.

---

# 15. Binary Reader

Implement a bounds-checked binary reader.

Useful operations:

```rust
read_u8
read_i8
read_u16_le
read_i16_le
read_u32_le
read_i32_le
read_bytes
skip
position
remaining
```

All parsing failures should carry:

```text
file name
byte offset
section name
expected operation
save version
```

Example:

```text
run-003.sav: offset 0x17A42, MercProfiles:
expected 716 encoded bytes for profile 63, only 491 remain
```

This is important for diagnosing malformed files and reviewing a future format deliberately; it is not a requirement to support historical versions.

---

# 16. Architecture

Suggested crate structure:

```text
src/
  main.rs
  cli.rs

  save/
    mod.rs
    reader.rs
    header.rs
    parser.rs
    version.rs
    encryption.rs
    sections.rs

  profile/
    mod.rs
    parser.rs
    ids.rs

  sector.rs
  output.rs
```

Keep binary format parsing separate from CLI formatting.

Preferred public model:

```rust
pub struct SaveAnalysis {
    pub header: SaveHeader,
    pub profiles: Vec<MercProfile>,
}
```

This permits future use as a Rust library.

---

# 17. Dependency and Platform Policy

The project must compile and run natively on:

- Linux
- Windows 10
- Windows 11

Use portable Rust and cross-platform crates. Do not depend on Unix-only APIs, shell commands, path conventions, or filesystem behavior at runtime. Treat input paths as platform-native paths, including Windows drive-letter and backslash paths. CI should build and test on both a current Linux runner and a current Windows runner.

Prefer a small dependency set.

Reasonable crates:

```text
clap
serde
serde_json
thiserror
anyhow
```

Use either `anyhow` or a structured custom error strategy appropriately.

Avoid bringing in a full game engine or C++ FFI merely to parse saves.

No runtime dependency on JA2 Stracciatella should be required.

---

# 18. Source-Derived Constants

Do not manually duplicate unexplained constants if they can be generated or clearly sourced.

For every important format constant in Rust, add a comment pointing to its Stracciatella origin.

Example:

```rust
// Stracciatella: SAVED_GAME_HEADER::ON_DISK_SIZE
const SAVE_HEADER_SIZE: usize = 432;
```

For complicated layouts such as `MERCPROFILESTRUCT`, mirror the order in `ExtractMercProfile()`.

That function should be regarded as authoritative for decoding a profile.

---

# 19. Avoid Parsing Unnecessary Fields

For the first implementation, it is not necessary to create Rust fields for all 716 bytes of `MERCPROFILESTRUCT`.

A targeted decoder is acceptable if it:

1. follows the exact field sequence,
2. skips known ranges structurally,
3. extracts at least:
   - profile name
   - nickname if available
   - `sSector.x`
   - `sSector.y`
   - `sSector.z`
   - status fields necessary to determine whether the profile is meaningfully placed
4. verifies that exactly the expected serialized profile size was consumed.

Example pattern:

```rust
let start = reader.position();

let name = ...;
let nickname = ...;

reader.skip(...);

// exact fields in ExtractMercProfile order
let sector_x = reader.read_u16_le()?;
let sector_y = reader.read_u16_le()?;

// continue following source layout

let sector_z = reader.read_i8()?;

// ...

assert_consumed(start, MERC_PROFILE_SIZE)?;
```

Do not jump directly to guessed offsets unless those offsets are accompanied by tests derived from the authoritative layout.

---

# 20. Determining Section Boundaries

The hardest part is reaching `SaveMercProfiles()` correctly.

Do not solve this by looking for recognizable bytes.

Use `SaveGame()` and `LoadSavedGame()` to model the serialized stream.

At the time of writing, the save process contains sections conceptually including:

```text
header
tactical status
game clock
strategic events
laptop info
merc profiles
soldier structures
finance/history/files data
email
strategic info
underground sectors
squads
strategic movement groups
map temp files
quests
opponent lists
map messages
NPC info
key table
temporary NPC quote array
arms dealer inventory
general info
...
save states
```

This is illustrative.

The implementation model must verify the current exact order against `SaveGame()`.

---

# 21. Development Strategy

Implement incrementally.

## Milestone 1 — Header

Given arbitrary supplied saves:

```text
ja2-savegame inspect foo.sav
```

correctly prints:

```text
save version
game version
description
day/time
sector
world-loaded state
```

Verify against Stracciatella's own save/load screen where practical.

## Milestone 2 — Reach merc profiles

Parse or safely skip all sections preceding `SaveMercProfiles()`.

Report the byte offset at which the profile array begins in debug mode.

## Milestone 3 — Decode profile encryption

Port the Stracciatella encryption/obfuscation logic.

Successfully parse plausible profile names and checksums.

## Milestone 4 — NPC locations

Output strategic locations for all appropriate NPCs.

This is the first feature-complete milestone.

---

# 22. Debug Mode

Provide:

```text
-v
-vv
```

or:

```text
--debug
```

Useful output:

```text
0x00000000 header
0x000001B0 tactical_status
0x00000ABC game_clock
...
0x00012345 merc_profiles
...
```

For each section optionally show bytes consumed:

```text
MercProfiles:
  start: 0x12345
  end:   0x45678
  size:  ...
```

This will make compatibility work much easier.

---

# 23. Validation

The parser must validate assumptions aggressively.

Examples:

- valid header
- supported save version
- valid profile block count
- every profile parser consumes exactly its defined on-disk size
- sector x/y either valid or recognized sentinel/unplaced values
- no file reads exceed bounds
- all expected merc profiles are reached and consume exactly their supported record sizes

Where Stracciatella stores checksums, verify them if practical.

Profile parsing should use any available profile checksum as an additional correctness signal.

---

# 24. Testing With Supplied Savegame Files

The test fixture corpus may contain multiple real `.sav` files, but every parser and CLI invocation must process them one at a time.

Successful fixture-parsing tests must use saves within the explicitly supported save-version set. Separate negative tests should verify clear rejection of unsupported versions, layouts, and attempts to pass multiple input files. Across independently parsed fixtures, tests must not assume every save has the same:

```text
save version
game version
NPC locations
number of mercs
current sector
world-loaded state
file size
```

Use the fixture set to test structural robustness, invoking the parser separately for each save.

Useful tests:

```text
each supplied file parses independently without panic
header values are plausible
profile section decrypts successfully
known NPC names occur
sector values fall into valid/sentinel ranges
```

When independently parsed fixtures disagree, treat the differences as possible game-state variation rather than a parser failure.

---

# 25. Cross-Validation

Where possible, use Stracciatella itself as an oracle during development.

For example:

1. load a supplied save in Stracciatella
2. inspect known NPC location in game/debug UI
3. compare with parser output

The final CLI must remain standalone.

Do not make runtime correctness depend on launching the game.

---

# 26. Non-Goals

The initial version does **not** need to:

- render the JA2 map
- reproduce gameplay
- run Stracciatella
- reconstruct player actions
- accept multiple save paths in one invocation or provide a multi-save `compare` command
- modify saves
- write saves
- support JA2 1.13
- support save versions 101 or earlier
- support legacy 688-byte Stracciatella Linux saves
- support German-edition save encoding or auto-detect game edition
- guarantee authoritative NPC/RPC classification for arbitrary mod-defined profiles
- parse sections after the merc-profile array merely to reach the end of the file
- decode every field of every serialized structure

Focus on reliable read-only analysis.

---

# 27. Deliverables

Produce:

```text
Cargo.toml
src/...
README.md
tests/...
```

README must contain Linux and Windows build instructions, including:

```text
cargo build --release

ja2-savegame inspect file.sav
ja2-savegame inspect file.sav --json
ja2-savegame inspect file.sav --npc Hamous --npc Skyrider
```

Also document:

- the exact supported English portable save versions
- explicit non-support for historical, old-Linux, German-edition, and unknown future versions
- known parser limitations, including stock-only authoritative NPC/RPC classification
- the pinned Stracciatella source commit and provenance of binary-layout constants

---

# 28. Acceptance Criteria

The implementation is considered successful when:

1. It compiles and runs as a normal Rust project on Linux, Windows 10, and Windows 11.

2. CI builds and tests the project on both Linux and Windows.

3. Each invocation accepts exactly one supplied Stracciatella `.sav` file in an explicitly supported English portable format; multiple input paths are rejected as a CLI usage error.

4. It parses the save format structurally rather than using offsets learned from one example.

5. For a supported file it extracts strategic NPC locations directly from saved `MERCPROFILESTRUCT` data.

6. Named NPC lookup works for at least:
   - Hamous
   - Skyrider
   - Devin
   - Carmen
   - Micky

   provided those profiles exist in the source/game version.

7. It emits both JA2 sector notation and raw coordinates.

8. JSON output is available.

9. Invalid or unsupported saves return informative errors rather than panicking.

10. No runtime Stracciatella installation is required.

11. No NPC position or binary offset is hard-coded from a particular sample save.

---

# 29. Critical Source Facts Already Established

The following observations have already been verified against Stracciatella source and should be used as starting points.

### Save header

`SAVED_GAME_HEADER` defines a normal on-disk size of:

```text
432 bytes
```

An old Stracciatella Linux representation is 688 bytes, but it is explicitly unsupported and should only be recognized well enough to reject clearly.

### NPC locations

`SaveMercProfiles()` serializes the full `gMercProfiles` array.

`ExtractMercProfile()` explicitly reads:

```text
p.sSector.x
p.sSector.y
...
p.sSector.z
```

Therefore NPC strategic locations are persisted in the `.sav`.

### Profile encoding

`SaveMercProfiles()` serializes a profile and then passes the bytes through:

```text
NewJA2EncryptedFileWrite
```

so a standalone parser must reproduce the corresponding decoding logic.

These facts make a standalone parser practical.

---

# 30. Instruction to the Implementing Coding Model

Do not guess the remaining binary format.

You have:

1. real `.sav` fixture files supplied by the user, each parsed independently, and
2. the complete open-source Stracciatella implementation.

Use the saves as test data and Stracciatella as the binary-format specification.

When an offset or size is unclear:

1. locate the corresponding `Save*` function,
2. locate its matching `Load*` function,
3. inspect its data structures and version conditions,
4. implement that behavior,
5. verify it independently against each supplied fixture.

Prefer a correct partial parser over a brittle full parser based on inferred offsets.

The first target to optimize for is:

```text
$ ja2-savegame inspect savegame.sav --npc Hamous --npc Skyrider --npc Devin

NPCs
  Hamous      D9       (4,9,0)
  Skyrider    B15      (2,15,0)
  Devin       C6       (3,6,0)
```

Once that works robustly for each supplied fixture independently, keep the focus on reliable NPC-placement reporting.

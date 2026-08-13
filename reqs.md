# JA2 Stracciatella Replay/Save Analyzer

## 1. Objective

Build a standalone Rust command-line tool that analyzes Jagged Alliance 2 Stracciatella `.sav` files.

These files may be referred to by the user as **replays**, but they are JA2/Stracciatella saved-game files rather than an event-stream replay format.

The primary purpose is to inspect RNG-dependent game state without launching JA2.

### Primary requirement

For each supplied `.sav` file, output the current strategic locations of NPCs.

At minimum, support named NPCs whose locations are stored in `MERCPROFILESTRUCT`, including examples such as:

- Hamous
- Skyrider
- Devin
- Carmen
- Micky
- Gabby

Do not hard-code this list as the only supported NPCs. Ideally output all profiles representing non-player NPCs for which a meaningful location exists.

### Secondary requirement

Extract as much useful RNG state as is serialized in the save, especially:

- save-header random value
- `guiPreRandomIndex`
- all 256 `guiPreRandomNums`
- any other serialized RNG state discoverable in current Stracciatella saves

The tool does **not** need to reconstruct historical RNG calls that are not present in the save.

---

# 2. Authoritative Reference Implementation

Use JA2 Stracciatella source code as the specification for the binary format.

Repository:

```text
https://github.com/ja2-stracciatella/ja2-stracciatella
```

Do not infer binary layouts from one supplied save if the corresponding Stracciatella serialization/deserialization code exists.

The implementation should inspect the repository source directly while being built.

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
SavePreRandomNumbersToSaveGameFile
LoadPreRandomNumbersFromSaveGameFile
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

There is also an old Stracciatella Linux format:

```text
688 bytes
```

The parser should use the same detection logic as:

```text
ExtractSavedGameHeaderFromFile
ParseSavedGameHeader
```

rather than simply assuming 432 bytes.

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
uiRandom
uiSaveStateSize
```

`uiRandom` is useful RNG-related information and must be reported.

Stracciatella creates it during save with conceptually:

```cpp
header.uiRandom = Random(RAND_MAX);
```

Note that this value is **not necessarily the full state of the modern RNG engine**. Report it accurately as the save-header random value rather than claiming it is a seed unless the source proves otherwise.

---

# 5. Top-Level Parsing Strategy

The parser must not search for byte signatures and assume fixed absolute offsets into the complete file.

Instead:

1. Parse the save header.
2. Obtain `uiSavedGameVersion`.
3. Follow the same section order and version conditions used by `LoadSavedGame()`.
4. Implement readers/skippers for enough preceding sections to reach the required sections.
5. Track the current byte offset.
6. Parse the required sections structurally.
7. Reject impossible sizes/counts instead of reading outside the file.

The implementation does not initially need semantic models for every save section.

For sections irrelevant to this tool, it is acceptable to implement:

```rust
fn skip_section(...)
```

provided the size can be derived safely from the actual serialized format.

Do not copy magic offsets from a sample `.sav`.

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

Do not assume 716 is valid for every historical format; inspect the source constants and version/old-Linux handling.

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
- make encryption-set calculation depend on parsed save metadata exactly as Stracciatella does

Do not describe this internally as strong cryptography. Treat it as the save encoding/obfuscation layer.

---

# 8. NPC Identification

A profile array index corresponds to a JA2 profile ID.

Resolve IDs to names from Stracciatella source/content rather than maintaining an undocumented manually invented table.

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

Exact profile ID integer width can follow the source representation.

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

A profile may not currently have a meaningful strategic location.

Distinguish these cases where possible:

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
```

to output every parsed profile, including profiles without valid strategic locations.

---

# 11. PreRandom State

Stracciatella's `Random.h` defines:

```cpp
#define MAX_PREGENERATED_NUMS 256

extern UINT32 guiPreRandomIndex;
extern UINT32 guiPreRandomNums[MAX_PREGENERATED_NUMS];
```

These values are serialized directly by:

```text
SavePreRandomNumbersToSaveGameFile
```

The on-disk payload is conceptually:

```text
UINT32 guiPreRandomIndex
UINT32 guiPreRandomNums[256]
```

Thus the section contains:

```text
4 + 256 * 4 = 1028 bytes
```

assuming normal 32-bit little-endian `UINT32` serialization as used by the implementation.

Extract and expose:

```rust
struct PreRandomState {
    index: u32,
    values: [u32; 256],
}
```

Validate:

```text
index < 256
```

unless Stracciatella semantics explicitly permit another value.

Do not claim that these are all random numbers used by JA2.

They specifically represent JA2's **pre-generated anti-save-scumming random sequence** used by `PreRandom()` / `PreChance()`.

---

# 12. Other RNG Information

Inspect:

```text
src/sgp/Random.h
src/sgp/Random.cc
```

The current engine exposes at least:

```text
Random(...)
PreRandom(...)
Chance(...)
PreChance(...)
gRandomEngine
guiPreRandomIndex
guiPreRandomNums
```

`gRandomEngine` is currently an `std::mt19937`.

Determine whether its complete state is serialized anywhere in the save.

If yes:

- decode and expose it.

If no:

- explicitly report that it is not present.
- do not pretend that `uiRandom` is equivalent to the MT19937 state.

Potential JSON representation:

```json
{
  "rng": {
    "header_random": 32576,
    "pre_random": {
      "index": 123,
      "values": [...]
    },
    "engine_state": null
  }
}
```

---

# 13. CLI Interface

Binary name:

```text
ja2-replay
```

Primary command:

```text
ja2-replay inspect <FILE>...
```

It must support one or many files.

Examples:

```text
ja2-replay inspect replay1.sav
ja2-replay inspect replay1.sav replay2.sav replay3.sav
```

Useful options:

```text
--json
--pretty
--all-profiles
--npc NAME
--rng
--no-rng
--source-version
```

`--npc` should be repeatable:

```text
ja2-replay inspect *.sav \
  --npc Hamous \
  --npc Skyrider \
  --npc Devin
```

Case-insensitive matching is desirable.

---

# 14. Default Human-Readable Output

Example format:

```text
replay-001.sav

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

RNG
  header random:       32576
  PreRandom index:     87
  PreRandom values:    256 values
```

The displayed values above are examples only.

Never encode these locations as expected values.

---

# 15. Multi-File Comparison

Because the main use case involves inspecting multiple RNG outcomes, multi-file output should make differences easy to compare.

For multiple files, a compact mode is desirable:

```text
FILE              Hamous   Skyrider   Devin   Carmen
run-001.sav       D9       B15        C6      C13
run-002.sav       D8       C16        H10     C13
run-003.sav       D9       B15        H10     C5
```

Suggested command:

```text
ja2-replay compare *.sav --npc Hamous --npc Skyrider --npc Devin
```

JSON support is also required or strongly preferred:

```text
ja2-replay compare *.sav --json
```

---

# 16. JSON Output

JSON must be stable enough for scripts.

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
  ],
  "rng": {
    "header_random": 32576,
    "pre_random": {
      "index": 87,
      "values": []
    },
    "engine_state": null
  }
}
```

Again, numeric/profile values shown here are examples unless independently derived from Stracciatella source.

---

# 17. Version Compatibility

Do not build specifically for the provided sample files.

The user will supply one or more real replay/save files, potentially produced by different Stracciatella releases.

Required behavior:

1. Parse the header first.
2. Read `uiSavedGameVersion`.
3. Follow version gates from `LoadSavedGame()`.
4. Detect known legacy Stracciatella Linux save layout when applicable.
5. Support at least the versions represented by supplied test files.
6. Make adding additional save versions straightforward.
7. Fail explicitly on an unsupported format.

Example error:

```text
unsupported JA2 save version 87 while parsing section StrategicInfo
```

not:

```text
unexpected EOF
```

where the actual cause can be identified.

A central version-aware abstraction is recommended:

```rust
struct SaveContext {
    save_version: u32,
    stracciatella_linux_format: bool,
    encryption_set: ...,
}
```

---

# 18. Binary Reader

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

This is important while reverse-engineering additional save versions.

---

# 19. Architecture

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

  rng/
    mod.rs

  sector.rs
  output.rs
```

Keep binary format parsing separate from CLI formatting.

Preferred public model:

```rust
pub struct SaveAnalysis {
    pub header: SaveHeader,
    pub profiles: Vec<MercProfile>,
    pub rng: RngState,
}
```

This permits future use as a Rust library.

---

# 20. Dependency Policy

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

# 21. Source-Derived Constants

Do not manually duplicate unexplained constants if they can be generated or clearly sourced.

For every important format constant in Rust, add a comment pointing to its Stracciatella origin.

Example:

```rust
// Stracciatella: SAVED_GAME_HEADER::ON_DISK_SIZE
const SAVE_HEADER_SIZE: usize = 432;

// Stracciatella: MAX_PREGENERATED_NUMS in src/sgp/Random.h
const PRE_RANDOM_COUNT: usize = 256;
```

For complicated layouts such as `MERCPROFILESTRUCT`, mirror the order in `ExtractMercProfile()`.

That function should be regarded as authoritative for decoding a profile.

---

# 22. Avoid Parsing Unnecessary Fields

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

# 23. Determining Section Boundaries

The hardest part is reaching `SaveMercProfiles()` and later `SavePreRandomNumbersToSaveGameFile()` correctly.

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
PreRandom state
arms dealer inventory
general info
...
save states
```

This is illustrative.

The implementation model must verify the current exact order against `SaveGame()`.

---

# 24. Development Strategy

Implement incrementally.

## Milestone 1 — Header

Given arbitrary supplied saves:

```text
ja2-replay inspect foo.sav
```

correctly prints:

```text
save version
game version
description
day/time
sector
world-loaded state
header random
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

## Milestone 5 — PreRandom

Continue parsing through later sections and extract:

```text
guiPreRandomIndex
guiPreRandomNums[256]
```

## Milestone 6 — Multiple files

Implement compact comparison.

---

# 25. Debug Mode

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
0x001A3456 pre_random
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

# 26. Validation

The parser must validate assumptions aggressively.

Examples:

- valid header
- supported save version
- valid profile block count
- every profile parser consumes exactly its defined on-disk size
- sector x/y either valid or recognized sentinel/unplaced values
- PreRandom block contains exactly 256 values
- no file reads exceed bounds
- final parsing reaches a consistent end/save-state location

Where Stracciatella stores checksums, verify them if practical.

Profile parsing should use any available profile checksum as an additional correctness signal.

---

# 27. Testing With Supplied Replay Files

The implementation will be given multiple real `.sav` files.

Tests must not assume that every supplied file has the same:

```text
save version
game version
NPC locations
RNG values
number of mercs
current sector
world-loaded state
file size
```

Use the corpus to test structural robustness.

Useful tests:

```text
all supplied files parse without panic
header values are plausible
profile section decrypts successfully
known NPC names occur
sector values fall into valid/sentinel ranges
PreRandom index is plausible
exactly 256 PreRandom values are parsed
```

When files disagree, assume RNG-dependent game state differs rather than treating the disagreement as a parser failure.

---

# 28. Cross-Validation

Where possible, use Stracciatella itself as an oracle during development.

For example:

1. load a supplied save in Stracciatella
2. inspect known NPC location in game/debug UI
3. compare with parser output

The final CLI must remain standalone.

Do not make runtime correctness depend on launching the game.

---

# 29. Non-Goals

The initial version does **not** need to:

- render the JA2 map
- reproduce gameplay
- run Stracciatella
- replay player actions
- reconstruct RNG calls made before the save
- modify saves
- write saves
- support JA2 1.13 unless it happens to share a verified compatible format
- support every historical vanilla JA2 save immediately
- decode every field of every serialized structure

Focus on reliable read-only analysis.

---

# 30. Important Terminology

Be precise in code and documentation.

The user calls these files **replays**, but internally use terms such as:

```text
save
savegame
SaveAnalysis
```

unless an actual Stracciatella replay format is discovered separately.

Similarly:

```text
uiRandom
```

should be called:

```text
header_random
```

not automatically:

```text
seed
```

And:

```text
guiPreRandomNums
```

should be called:

```text
pre_random
```

rather than claiming they represent all game RNG.

---

# 31. Deliverables

Produce:

```text
Cargo.toml
src/...
README.md
tests/...
```

README must contain:

```text
cargo build --release

ja2-replay inspect file.sav
ja2-replay inspect *.sav --json
ja2-replay compare *.sav --npc Hamous --npc Skyrider
```

Also document:

- tested Stracciatella save versions
- unsupported versions
- known parser limitations
- provenance of binary-layout constants

---

# 32. Acceptance Criteria

The implementation is considered successful when:

1. It compiles as a normal Rust project.

2. It accepts one or multiple arbitrary supplied Stracciatella `.sav` files.

3. It parses the save format structurally rather than using offsets learned from one example.

4. For each supported file it extracts strategic NPC locations directly from saved `MERCPROFILESTRUCT` data.

5. Named NPC lookup works for at least:
   - Hamous
   - Skyrider
   - Devin
   - Carmen
   - Micky

   provided those profiles exist in the source/game version.

6. It emits both JA2 sector notation and raw coordinates.

7. It extracts the header `uiRandom`.

8. It extracts:
   - `guiPreRandomIndex`
   - all 256 `guiPreRandomNums`

9. JSON output is available.

10. Multiple saves can be compared conveniently.

11. Invalid or unsupported saves return informative errors rather than panicking.

12. No runtime Stracciatella installation is required.

13. No NPC position or binary offset is hard-coded from a particular sample save.

---

# 33. Critical Source Facts Already Established

The following observations have already been verified against Stracciatella source and should be used as starting points.

### Save header

`SAVED_GAME_HEADER` defines a normal on-disk size of:

```text
432 bytes
```

and an old Stracciatella Linux representation of:

```text
688 bytes
```

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

### RNG header value

The save header contains:

```text
uiRandom
```

which is populated from:

```text
Random(RAND_MAX)
```

during save.

### Pre-generated RNG state

`Random.h` defines:

```text
MAX_PREGENERATED_NUMS = 256
guiPreRandomIndex
guiPreRandomNums[256]
```

and `SavePreRandomNumbersToSaveGameFile()` writes the index followed by the complete 256-entry array.

These facts make a standalone parser practical.

---

# 34. Instruction to the Implementing Coding Model

Do not guess the remaining binary format.

You have:

1. one or more real `.sav` files supplied by the user, and
2. the complete open-source Stracciatella implementation.

Use the saves as test data and Stracciatella as the binary-format specification.

When an offset or size is unclear:

1. locate the corresponding `Save*` function,
2. locate its matching `Load*` function,
3. inspect its data structures and version conditions,
4. implement that behavior,
5. verify it against all supplied files.

Prefer a correct partial parser over a brittle full parser based on inferred offsets.

The first target to optimize for is:

```text
$ ja2-replay compare *.sav --npc Hamous --npc Skyrider --npc Devin

FILE             Hamous    Skyrider    Devin
a.sav            ...
b.sav            ...
c.sav            ...
```

Once that works robustly across supplied saves, add full RNG-state reporting.

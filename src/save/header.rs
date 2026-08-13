use serde::Serialize;

use crate::{save::reader::Reader, sector::Sector};

use super::{ParseError, SupportedSaveVersion};

/// Stracciatella: `SAVED_GAME_HEADER::ON_DISK_SIZE`.
pub const SAVE_HEADER_SIZE: usize = 432;
/// Unsupported historical Stracciatella Linux representation.
const LEGACY_LINUX_HEADER_SIZE: usize = 688;

#[derive(Debug, Clone, Serialize)]
pub struct SaveHeader {
    pub save_version: SupportedSaveVersion,
    pub game_version: String,
    pub description: String,
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
    pub sector: Sector,
    pub mercs_on_team: u8,
    pub current_balance: i32,
    pub current_screen: u32,
    pub alternate_sector: bool,
    pub world_loaded: bool,
    pub load_screen_id: u8,
    #[serde(skip)]
    pub(crate) gun_nut: bool,
    #[serde(skip)]
    pub(crate) sci_fi: bool,
    #[serde(skip)]
    pub(crate) difficulty: u8,
    #[serde(skip)]
    pub(crate) random: u32,
    pub save_state_size: u32,
}

pub fn parse(reader: &mut Reader<'_>, bytes: &[u8]) -> Result<SaveHeader, ParseError> {
    if bytes.len() < SAVE_HEADER_SIZE {
        return Err(reader.error(format!(
            "expected {SAVE_HEADER_SIZE}-byte portable save header, only {} bytes available",
            bytes.len()
        )));
    }

    if looks_like_legacy_linux_header(bytes) {
        return Err(ParseError::LegacyLinux);
    }

    reader.set_section("Header");
    let raw_version = reader.read_u32_le()?;
    reader.set_save_version(raw_version);
    let save_version = SupportedSaveVersion::new(raw_version)?;
    let game_version = decode_bytes(reader.read_bytes(16, "expected game-version string")?);
    let description = decode_utf16_le(reader.read_bytes(256, "expected UTF-16 description")?);
    reader.skip(4, "expected obsolete header field")?;
    let day = reader.read_u32_le()?;
    let hour = reader.read_u8()?;
    let minute = reader.read_u8()?;
    let sector_x = reader.read_i16_le()?;
    let sector_y = reader.read_i16_le()?;
    let sector_z = reader.read_i8()?;
    let mercs_on_team = reader.read_u8()?;
    let current_balance = reader.read_i32_le()?;
    let current_screen = reader.read_u32_le()?;
    let alternate_sector = read_bool(reader, "alternate-sector flag")?;
    let world_loaded = read_bool(reader, "world-loaded flag")?;
    let load_screen_id = reader.read_u8()?;
    let gun_nut = read_bool(reader, "gun-nut option")?;
    let sci_fi = read_bool(reader, "sci-fi option")?;
    let difficulty = reader.read_u8()?;
    let _turn_time_limit = read_bool(reader, "turn-time-limit option")?;
    let _save_mode = reader.read_u8()?;
    reader.skip(7, "expected game-options reserved bytes")?;
    reader.skip(1, "expected header reserved byte")?;
    let random = reader.read_u32_le()?;
    let save_state_size = reader.read_u32_le()?;
    if save_state_size == 0 {
        return Err(reader.error("save-state size is zero in a version 102+ header"));
    }
    reader.skip(108, "expected trailing header reserved bytes")?;

    if reader.position() != SAVE_HEADER_SIZE {
        return Err(reader.error("portable header parser did not consume exactly 432 bytes"));
    }
    validate_header(
        reader,
        day,
        hour,
        minute,
        sector_x,
        sector_y,
        sector_z,
        current_balance,
        difficulty,
    )?;

    Ok(SaveHeader {
        save_version,
        game_version,
        description,
        day,
        hour,
        minute,
        sector: Sector::new(sector_x as u16, sector_y as u16, sector_z),
        mercs_on_team,
        current_balance,
        current_screen,
        alternate_sector,
        world_loaded,
        load_screen_id,
        gun_nut,
        sci_fi,
        difficulty,
        random,
        save_state_size,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_header(
    reader: &Reader<'_>,
    day: u32,
    hour: u8,
    minute: u8,
    x: i16,
    y: i16,
    z: i8,
    current_balance: i32,
    difficulty: u8,
) -> Result<(), ParseError> {
    let sector_valid = (1..=16).contains(&x) && (1..=16).contains(&y) && (0..=3).contains(&z);
    let pregame = x == 0 && y == 0 && z == -1;
    if day == 0 || hour > 23 || minute > 59 || current_balance < 0 || (!sector_valid && !pregame) {
        return Err(reader.error(format!(
            "invalid portable header values (day {day}, time {hour:02}:{minute:02}, sector {x},{y},{z})"
        )));
    }
    if !(1..=3).contains(&difficulty) {
        return Err(reader.error(format!("invalid difficulty value {difficulty}")));
    }
    Ok(())
}

fn read_bool(reader: &mut Reader<'_>, name: &str) -> Result<bool, ParseError> {
    match reader.read_u8()? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(reader.error(format!("invalid {name} value {value}; expected 0 or 1"))),
    }
}

fn decode_bytes(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_owned()
}

pub(crate) fn decode_utf16_le(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|&unit| unit != 0)
        .collect();
    String::from_utf16_lossy(&units).trim().to_owned()
}

fn looks_like_legacy_linux_header(bytes: &[u8]) -> bool {
    if bytes.len() < LEGACY_LINUX_HEADER_SIZE {
        return false;
    }
    // `ParseSavedGameHeader(..., stracLinuxFormat=true)`: the 128-code-point
    // description occupies 512 bytes, moving these fields by 256 bytes.
    let version = u32_at(bytes, 0);
    let day = u32_at(bytes, 536);
    let x = i16_at(bytes, 542);
    let y = i16_at(bytes, 544);
    let z = bytes[546] as i8;
    let balance = i32_at(bytes, 548);
    let difficulty = bytes[561];
    let sector_valid = (1..=16).contains(&x) && (1..=16).contains(&y) && (0..=3).contains(&z);
    let pregame = x == 0 && y == 0 && z == -1;
    version > 0
        && day > 0
        && balance >= 0
        && (sector_valid || pregame)
        && (1..=3).contains(&difficulty)
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("checked header"),
    )
}
fn i32_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("checked header"),
    )
}
fn i16_at(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("checked header"),
    )
}

use serde::Serialize;

use crate::sector::Sector;

use super::{reader::Reader, ParseError, SectionTrace};

const ITEM_TEMP_FILE: u32 = 0x1000_0000;
// Tactical_Save.cc: sectorFlagBits. This is file order, not numeric bit order.
const TEMP_FILE_FLAGS: [u32; 10] = [
    ITEM_TEMP_FILE,
    0x2000_0000,
    0x4000_0000,
    0x8000_0000,
    0x0100_0000,
    0x0200_0000,
    0x0400_0000,
    0x0800_0000,
    0x0010_0000,
    0x0020_0000,
];
const WORLD_ITEM_SIZE: usize = 52;

/// Saved ground inventory, not character inventory. `None` means no item file
/// was saved. `Some([])` means the saved item file contains no records.
#[derive(Debug, Clone, Serialize)]
pub struct SectorItems {
    pub sector: Sector,
    pub flags: u32,
    pub items: Option<Vec<SavedWorldItem>>,
}

impl SectorItems {
    /// Includes hidden items. Removed records do not contribute to the count.
    pub fn count(&self, item_index: u16) -> Option<u64> {
        self.items.as_ref().map(|items| {
            items
                .iter()
                .filter(|item| item.exists && item.item_index == item_index)
                .map(|item| u64::from(item.quantity))
                .sum()
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SavedWorldItem {
    pub exists: bool,
    pub grid_no: i16,
    pub level: u8,
    pub item_index: u16,
    pub quantity: u8,
    pub flags: u16,
    pub visible: i8,
}

/// SaveLoadGame.cc: traversal immediately after LoadSoldierStructure, for
/// portable versions 102 and 103. All variable sections are length/count led.
pub(super) fn parse_sector_items(
    reader: &mut Reader<'_>,
    sections: &mut Vec<SectionTrace>,
) -> Result<Vec<SectorItems>, ParseError> {
    for name in ["finances_file", "history_file", "laptop_files"] {
        let start = reader.position();
        reader.set_section(name);
        read_file(reader)?;
        trace(sections, name, start, reader);
    }

    let start = reader.position();
    reader.set_section("Email");
    let emails = reader.read_u32_le()? as usize;
    check_records(reader, emails, 48, "emails")?;
    for _ in 0..emails {
        let subject_size = reader.read_u32_le()? as usize;
        reader.skip(subject_size, "expected email subject bytes")?;
        reader.skip(44, "expected 44-byte email record")?;
    }
    trace(sections, "email", start, reader);

    let start = reader.position();
    reader.set_section("StrategicInfo");
    // StrategicMap.cc / LoadSaveStrategicMapElement.cc: 18*18 map cells.
    reader.skip(18 * 18 * 41, "expected strategic map records")?;
    let mut sectors = Vec::with_capacity(256);
    // LoadSaveSectorInfo.cc: 116 bytes, flags at offset 0; row-major grid.
    for y in 1..=16 {
        for x in 1..=16 {
            let flags = reader.read_u32_le()?;
            reader.skip(112, "expected rest of 116-byte sector record")?;
            sectors.push(SectorItems {
                sector: Sector::new(x, y, 0),
                flags,
                items: None,
            });
        }
    }
    reader.skip(18 * 18 + 1, "expected SAM control padding and Orta state")?;
    trace(sections, "strategic_info", start, reader);

    let start = reader.position();
    reader.set_section("UndergroundSectorInfo");
    let underground_count = reader.read_u32_le()? as usize;
    check_records(reader, underground_count, 72, "underground sectors")?;
    // Queen_Command.cc preserves list order. It is also temp file order.
    for _ in 0..underground_count {
        let flags = reader.read_u32_le()?;
        let x = reader.read_u8()?;
        let y = reader.read_u8()?;
        let z = reader.read_u8()?;
        let sector = Sector::new(u16::from(x), u16::from(y), z as i8);
        if !sector.is_valid() || z == 0 {
            return Err(reader.error(format!("invalid underground sector ({x}, {y}, {z})")));
        }
        if sectors.iter().any(|entry| entry.sector == sector) {
            return Err(reader.error("duplicate underground sector"));
        }
        reader.skip(65, "expected rest of 72-byte underground sector record")?;
        sectors.push(SectorItems {
            sector,
            flags,
            items: None,
        });
    }
    trace(sections, "underground_sector_info", start, reader);

    let start = reader.position();
    reader.set_section("Squads");
    // Squads.cc: 20 squads * 6 records * 12 bytes, then 20 movement IDs.
    reader.skip(20 * 6 * 12 + 20, "expected squad records and movement IDs")?;
    trace(sections, "squads", start, reader);

    let start = reader.position();
    reader.set_section("StrategicMovementGroups");
    let groups = reader.read_u32_le()? as usize;
    check_records(reader, groups, 88, "movement groups")?;
    for _ in 0..groups {
        let group = reader.read_bytes(84, "expected 84-byte movement group")?;
        if group[1] != 0 {
            // Player groups with size zero have no player-list count field.
            if group[5] != 0 {
                skip_counted(reader, 4, "player profile IDs")?;
            }
        } else {
            reader.skip(29, "expected 29-byte enemy group")?;
        }
        skip_counted(reader, 8, "waypoints")?;
    }
    reader.skip(32, "expected movement group unique ID mask")?;
    trace(sections, "strategic_movement_groups", start, reader);

    let start = reader.position();
    reader.set_section("MapTempFiles");
    for sector in &mut sectors {
        for flag in TEMP_FILE_FLAGS {
            if sector.flags & flag == 0 {
                continue;
            }
            let payload = read_file(reader)?;
            if flag == ITEM_TEMP_FILE {
                sector.items = Some(parse_items(reader, payload)?);
            }
        }
    }
    trace(sections, "map_temp_files", start, reader);
    Ok(sectors)
}

fn trace(sections: &mut Vec<SectionTrace>, name: &'static str, start: usize, reader: &Reader<'_>) {
    sections.push(SectionTrace {
        name,
        start,
        end: reader.position(),
    });
}

fn read_file<'a>(reader: &mut Reader<'a>) -> Result<&'a [u8], ParseError> {
    // SaveLoadGame.cc: LoadFileFromSavedGame, size excludes its u32 prefix.
    let size = reader.read_u32_le()? as usize;
    reader.read_bytes(size, "expected embedded file payload")
}

fn check_records(
    reader: &Reader<'_>,
    count: usize,
    stride: usize,
    name: &str,
) -> Result<usize, ParseError> {
    let size = count
        .checked_mul(stride)
        .ok_or_else(|| reader.error(format!("{name} count {count} overflows")))?;
    if size > reader.remaining() {
        return Err(reader.error(format!(
            "expected {count} {name} of at least {stride} bytes each; only {} bytes remain",
            reader.remaining()
        )));
    }
    Ok(size)
}

fn skip_counted(reader: &mut Reader<'_>, stride: usize, name: &str) -> Result<(), ParseError> {
    let count = reader.read_u32_le()? as usize;
    let size = check_records(reader, count, stride, name)?;
    reader.skip(size, format!("expected {name}"))
}

fn parse_items(reader: &Reader<'_>, payload: &[u8]) -> Result<Vec<SavedWorldItem>, ParseError> {
    // Tactical_Save.cc: LoadWorldItemsFromTempItemFile. World_Items.h fixes
    // WORLDITEM at 52 bytes; Item_Types.h fixes its OBJECTTYPE at 36 bytes.
    if payload.len() < 4 {
        return Err(reader.error("item temp file needs a 4-byte record count"));
    }
    let count = u32::from_le_bytes(payload[..4].try_into().expect("four bytes")) as usize;
    let required = count
        .checked_mul(WORLD_ITEM_SIZE)
        .and_then(|size| size.checked_add(4))
        .ok_or_else(|| reader.error("item temp file record count overflows"))?;
    if payload.len() != required {
        return Err(reader.error(format!(
            "item temp file declares {count} records: expected {required} bytes, got {}",
            payload.len()
        )));
    }
    Ok(payload[4..]
        .as_chunks::<WORLD_ITEM_SIZE>()
        .0
        .iter()
        .map(|record| SavedWorldItem {
            exists: record[0] != 0,
            grid_no: i16::from_le_bytes([record[2], record[3]]),
            level: record[4],
            item_index: u16::from_le_bytes([record[8], record[9]]),
            quantity: record[10],
            flags: u16::from_le_bytes([record[44], record[45]]),
            visible: record[47] as i8,
        })
        .collect())
}

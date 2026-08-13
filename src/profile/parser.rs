use serde::Serialize;

use crate::{
    save::{header::decode_utf16_le, ParseError},
    sector::Sector,
};

use super::ids::{stock_npc_or_rpc, StockProfileType};

/// Stracciatella: `MERC_PROFILE_SIZE` in LoadSaveMercProfile.h.
pub const MERC_PROFILE_SIZE: usize = 716;
/// Stracciatella: `NUM_PROFILES` in Soldier_Profile_Type.h.
pub const NUM_PROFILES: usize = 170;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileType {
    Npc,
    Rpc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationState {
    Placed,
    NotCurrentlyPlaced,
    Dead,
    Unavailable,
    Recruited,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct MercProfile {
    pub profile_id: u8,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_type: Option<ProfileType>,
    pub sector: Sector,
    pub location_state: LocationState,
    pub merc_status: i8,
    pub recruited: bool,
    pub use_insertion_info: bool,
    pub grid_no: i16,
    pub strategic_insertion_code: u8,
    pub strategic_insertion_data: u16,
}

impl MercProfile {
    pub fn is_stock_npc_or_rpc(&self) -> bool {
        self.profile_type.is_some()
    }

    pub fn has_meaningful_location(&self) -> bool {
        self.sector.is_valid()
            && !matches!(
                self.location_state,
                LocationState::NotCurrentlyPlaced | LocationState::Dead
            )
    }

    pub fn matches_name(&self, needle: &str) -> bool {
        let needle = needle.trim().to_lowercase();
        self.name.to_lowercase() == needle
            || self
                .nickname
                .as_deref()
                .is_some_and(|name| name.to_lowercase() == needle)
            || self
                .canonical_name
                .is_some_and(|name| name.to_lowercase() == needle)
    }

    pub fn display_name(&self) -> &str {
        if !self.name.is_empty() {
            &self.name
        } else if let Some(nickname) = &self.nickname {
            nickname
        } else {
            self.canonical_name.unwrap_or("(unnamed)")
        }
    }
}

/// Targeted decoder mirroring the complete `ExtractMercProfile` field order.
/// Every skip below corresponds to a documented serialized field range.
pub fn parse_profile(decoded: &[u8], profile_id: u8) -> Result<MercProfile, ParseError> {
    if decoded.len() != MERC_PROFILE_SIZE {
        return Err(ParseError::Profile {
            profile_id,
            message: format!(
                "expected exactly {MERC_PROFILE_SIZE} decoded bytes, got {}",
                decoded.len()
            ),
        });
    }
    let mut p = ProfileReader::new(decoded, profile_id);

    let name = decode_utf16_le(p.take(60)?);
    let nickname = nonempty(decode_utf16_le(p.take(20)?));
    p.skip(28)?;
    p.skip(1)?; // face
    p.skip(120)?; // palette strings
    p.skip(2)?; // sex, armour attractiveness
    let misc_flags_2 = p.u8()?;
    p.skip(1)?; // evolution
    let misc_flags = p.u8()?;
    p.skip(2)?; // sexist, learn-to-hate
    p.skip(2)?;
    p.skip(2)?; // quote, death rate
    p.skip(2)?;
    p.skip(18)?; // stat gains
    p.skip(1)?; // body type
    let medical = p.i8()?;
    p.skip(8)?; // face coordinates
    p.skip(10)?;
    p.skip(8)?; // expression frequencies
    let sector_x = p.u16()?;
    let sector_y = p.u16()?;
    p.skip(4)?; // available day
    let strength = p.i8()?;
    let life_max = p.i8()?;
    p.skip(11)?; // remaining stat deltas
    p.skip(1)?;
    p.skip(14)?; // career counters
    p.skip(4)?; // leadership/strength gains
    p.skip(6)?; // body flags, salary
    let life = p.i8()?; // Used by the checksum; dead state is explicit bMercStatus.
    let dexterity = p.i8()?;
    p.skip(3)?; // personality, skill, reputation
    let explosive = p.i8()?;
    p.skip(1)?; // second skill
    p.skip(1)?; // leadership
    p.skip(10)?; // buddies/hated
    let exp_level = p.i8()?;
    let marksmanship = p.i8()?;
    p.skip(1)?;
    p.skip(1)?; // wisdom
    p.skip(2)?;
    p.skip(19)?; // inventory status
    let inventory_counts = p.take(19)?.to_vec();
    p.skip(8)?; // approach factors
    p.skip(1)?; // gun attractiveness
    let agility = p.i8()?;
    let use_insertion_info = p.u8()? != 0;
    p.skip(1)?;
    let grid_no = p.i16()?;
    p.skip(1)?; // quote action
    let mechanical = p.i8()?;
    p.skip(3)?; // undroppable, room range start
    p.skip(1)?;
    let inventory: Vec<u16> = (0..19).map(|_| p.u16()).collect::<Result<_, _>>()?;
    p.skip(20)?;
    p.skip(24)?; // stat chances
    p.skip(24)?; // stat successes
    let strategic_insertion_code = p.u8()?;
    p.skip(2)?; // room range end
    p.skip(4)?;
    p.skip(10)?; // quote/race/etc.
    p.skip(1)?;
    p.skip(8)?; // weekly salaries
    p.skip(2)?;
    p.skip(2)?;
    p.skip(3)?;
    p.skip(4)?; // approach val
    p.skip(12)?; // approach mod
    p.skip(2)?; // town
    p.skip(1)?;
    p.skip(2)?; // optional gear
    p.skip(75)?; // opinions
    p.skip(1)?; // approached
    let merc_status = p.i8()?;
    p.skip(5)?;
    p.skip(2)?;
    p.skip(5)?;
    p.skip(2)?;
    p.skip(2)?;
    let sector_z = p.i8()?;
    let strategic_insertion_data = p.u16()?;
    p.skip(4)?;
    p.skip(4)?; // balance
    p.skip(2)?;
    p.skip(2)?;
    p.skip(4)?; // money
    p.skip(4)?; // NPC data and usage counters
    p.skip(4)?; // precedent quote bitfield
    let stored_checksum = p.u32()?;
    p.skip(2)?;
    p.skip(2)?;
    p.skip(8)?;
    p.skip(4)?;
    if p.position != MERC_PROFILE_SIZE {
        return Err(p.error(format!(
            "field decoder consumed {} bytes instead of {MERC_PROFILE_SIZE}",
            p.position
        )));
    }

    let calculated_checksum = profile_checksum(
        life,
        life_max,
        agility,
        dexterity,
        strength,
        marksmanship,
        medical,
        mechanical,
        explosive,
        exp_level,
        &inventory,
        &inventory_counts,
    );
    if calculated_checksum != stored_checksum {
        return Err(ParseError::Profile {
            profile_id,
            message: format!(
                "profile decoding/checksum validation failed: stored 0x{stored_checksum:08X}, calculated 0x{calculated_checksum:08X}; input may use an unsupported edition or format"
            ),
        });
    }

    let stock = stock_npc_or_rpc(profile_id);
    let profile_type = stock.map(|s| match s.profile_type {
        StockProfileType::Npc => ProfileType::Npc,
        StockProfileType::Rpc => ProfileType::Rpc,
    });
    let sector = Sector::new(sector_x, sector_y, sector_z);
    let recruited = misc_flags & 0x01 != 0;
    let location_state = if merc_status == -5 {
        LocationState::Dead
    } else if recruited {
        LocationState::Recruited
    } else if merc_status > 0 || (-8..=-1).contains(&merc_status) {
        // Source-defined nonzero statuses other than MERC_IS_DEAD (-5)
        // describe a merc who is unavailable or away.
        LocationState::Unavailable
    } else if !sector.is_valid() || misc_flags_2 & 0x01 != 0 {
        LocationState::NotCurrentlyPlaced
    } else if merc_status == 0 {
        LocationState::Placed
    } else {
        LocationState::Unknown
    };

    Ok(MercProfile {
        profile_id,
        name,
        nickname,
        canonical_name: stock.map(|s| s.canonical_name),
        profile_type,
        sector,
        location_state,
        merc_status,
        recruited,
        use_insertion_info,
        grid_no,
        strategic_insertion_code,
        strategic_insertion_data,
    })
}

#[allow(clippy::too_many_arguments)]
fn profile_checksum(
    life: i8,
    life_max: i8,
    agility: i8,
    dexterity: i8,
    strength: i8,
    marksmanship: i8,
    medical: i8,
    mechanical: i8,
    explosive: i8,
    exp_level: i8,
    inventory: &[u16],
    inventory_counts: &[u8],
) -> u32 {
    let mut sum = 1u32;
    sum = sum.wrapping_add_signed(1 + i32::from(life));
    sum = sum.wrapping_mul((1 + i32::from(life_max)) as u32);
    sum = sum.wrapping_add_signed(1 + i32::from(agility));
    sum = sum.wrapping_mul((1 + i32::from(dexterity)) as u32);
    sum = sum.wrapping_add_signed(1 + i32::from(strength));
    sum = sum.wrapping_mul((1 + i32::from(marksmanship)) as u32);
    sum = sum.wrapping_add_signed(1 + i32::from(medical));
    sum = sum.wrapping_mul((1 + i32::from(mechanical)) as u32);
    sum = sum.wrapping_add_signed(1 + i32::from(explosive));
    sum = sum.wrapping_mul((1 + i32::from(exp_level)) as u32);
    for &item in inventory {
        sum = sum.wrapping_add(u32::from(item));
    }
    for &count in inventory_counts {
        sum = sum.wrapping_add(u32::from(count));
    }
    sum
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

struct ProfileReader<'a> {
    bytes: &'a [u8],
    position: usize,
    profile_id: u8,
}

impl<'a> ProfileReader<'a> {
    fn new(bytes: &'a [u8], profile_id: u8) -> Self {
        Self {
            bytes,
            position: 0,
            profile_id,
        }
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], ParseError> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| self.error("profile offset overflow"))?;
        if end > self.bytes.len() {
            return Err(self.error(format!(
                "expected {count} bytes at profile offset 0x{:X}",
                self.position
            )));
        }
        let result = &self.bytes[self.position..end];
        self.position = end;
        Ok(result)
    }
    fn skip(&mut self, count: usize) -> Result<(), ParseError> {
        self.take(count).map(|_| ())
    }
    fn u8(&mut self) -> Result<u8, ParseError> {
        Ok(self.take(1)?[0])
    }
    fn i8(&mut self) -> Result<i8, ParseError> {
        Ok(self.u8()? as i8)
    }
    fn u16(&mut self) -> Result<u16, ParseError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn i16(&mut self) -> Result<i16, ParseError> {
        Ok(self.u16()? as i16)
    }
    fn u32(&mut self) -> Result<u32, ParseError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes(b.try_into().expect("four bytes")))
    }
    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError::Profile {
            profile_id: self.profile_id,
            message: message.into(),
        }
    }
}

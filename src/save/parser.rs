use std::{fs, path::Path};

use serde::Serialize;

use crate::profile::{parse_profile, MercProfile, MERC_PROFILE_SIZE, NUM_PROFILES};

use super::{
    encryption,
    header::{self, SAVE_HEADER_SIZE},
    reader::Reader,
    ParseError, SaveHeader, SupportedSaveVersion,
};

const TACTICAL_STATUS_AND_SECTOR_SIZE: usize = 316 + 5;
const GAME_CLOCK_SIZE: usize = 62;
const STRATEGIC_EVENT_SIZE: usize = 28;
const LAPTOP_FIXED_SIZE: usize = 7_440;
const BOBBY_RAY_ORDER_SIZE: usize = 84;
const INSURANCE_PAYOUT_SIZE: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub struct SaveAnalysis {
    pub file: String,
    pub header: SaveHeader,
    pub profiles: Vec<MercProfile>,
    #[serde(skip)]
    pub sections: Vec<SectionTrace>,
}

#[derive(Debug, Clone)]
pub struct SectionTrace {
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
}

impl SectionTrace {
    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, Copy)]
struct SaveContext {
    save_version: SupportedSaveVersion,
    encryption_set: usize,
}

pub fn analyze_file(path: &Path) -> Result<SaveAnalysis, ParseError> {
    let bytes = fs::read(path).map_err(|source| ParseError::Io {
        file: path.to_path_buf(),
        source,
    })?;
    analyze_bytes(path, &bytes)
}

pub fn analyze_bytes(path: &Path, bytes: &[u8]) -> Result<SaveAnalysis, ParseError> {
    let mut reader = Reader::new(bytes, path);
    let header = header::parse(&mut reader, bytes)?;
    let context = SaveContext {
        save_version: header.save_version,
        encryption_set: encryption::calculate_set(&header),
    };
    let mut sections = vec![SectionTrace {
        name: "header",
        start: 0,
        end: SAVE_HEADER_SIZE,
    }];

    skip_fixed(
        &mut reader,
        &mut sections,
        "tactical_status",
        TACTICAL_STATUS_AND_SECTOR_SIZE,
    )?;
    skip_fixed(&mut reader, &mut sections, "game_clock", GAME_CLOCK_SIZE)?;
    skip_strategic_events(&mut reader, &mut sections)?;
    skip_laptop(&mut reader, &mut sections)?;

    let profile_start = reader.position();
    reader.set_section("MercProfiles");
    let required = NUM_PROFILES
        .checked_mul(MERC_PROFILE_SIZE)
        .expect("profile constants fit usize");
    if reader.remaining() < required {
        return Err(reader.error(format!(
            "expected {MERC_PROFILE_SIZE} encoded bytes for each of {NUM_PROFILES} profiles ({required} total), only {} remain",
            reader.remaining()
        )));
    }

    let mut profiles = Vec::with_capacity(NUM_PROFILES);
    for profile_id in 0..NUM_PROFILES {
        let record_start = reader.position();
        let encoded = reader.read_bytes(
            MERC_PROFILE_SIZE,
            format!("expected {MERC_PROFILE_SIZE} encoded bytes for profile {profile_id}"),
        )?;
        let decoded = encryption::decrypt(encoded, context.encryption_set);
        let profile =
            parse_profile(&decoded, profile_id as u8).map_err(|error| ParseError::Format {
                file: path.to_path_buf(),
                offset: record_start,
                section: "MercProfiles",
                operation: error.to_string(),
                save_version: Some(context.save_version.get()),
            })?;
        profiles.push(profile);
    }
    sections.push(SectionTrace {
        name: "merc_profiles",
        start: profile_start,
        end: reader.position(),
    });

    Ok(SaveAnalysis {
        file: path.to_string_lossy().into_owned(),
        header,
        profiles,
        sections,
    })
}

fn skip_fixed(
    reader: &mut Reader<'_>,
    sections: &mut Vec<SectionTrace>,
    name: &'static str,
    size: usize,
) -> Result<(), ParseError> {
    let start = reader.position();
    reader.set_section(name);
    reader.skip(size, format!("expected {size}-byte {name} section"))?;
    sections.push(SectionTrace {
        name,
        start,
        end: reader.position(),
    });
    Ok(())
}

fn skip_strategic_events(
    reader: &mut Reader<'_>,
    sections: &mut Vec<SectionTrace>,
) -> Result<(), ParseError> {
    let start = reader.position();
    reader.set_section("StrategicEvents");
    let count = reader.read_u32_le()? as usize;
    let size = count
        .checked_mul(STRATEGIC_EVENT_SIZE)
        .ok_or_else(|| reader.error(format!("strategic-event count {count} overflows")))?;
    reader.skip(
        size,
        format!("expected {count} strategic events of {STRATEGIC_EVENT_SIZE} bytes each"),
    )?;
    sections.push(SectionTrace {
        name: "strategic_events",
        start,
        end: reader.position(),
    });
    Ok(())
}

fn skip_laptop(
    reader: &mut Reader<'_>,
    sections: &mut Vec<SectionTrace>,
) -> Result<(), ParseError> {
    let start = reader.position();
    reader.set_section("Laptop");
    // Stracciatella Laptop.cc writes a fixed 7,440-byte buffer followed by two
    // separately gated vectors. Counts and gates are inside the fixed buffer.
    let fixed = reader.read_bytes(LAPTOP_FIXED_SIZE, "expected 7440-byte laptop data")?;
    let bobby_count = usize::from(fixed[7276]);
    let bobby_used = fixed[7277] != 0;
    let insurance_count = usize::from(fixed[7284]);
    let insurance_used = fixed[7285] != 0;

    if bobby_used {
        let size = bobby_count
            .checked_mul(BOBBY_RAY_ORDER_SIZE)
            .ok_or_else(|| reader.error("Bobby Ray order vector size overflow"))?;
        reader.skip(
            size,
            format!("expected {bobby_count} Bobby Ray orders of 84 bytes each"),
        )?;
    }
    if insurance_used {
        let size = insurance_count
            .checked_mul(INSURANCE_PAYOUT_SIZE)
            .ok_or_else(|| reader.error("insurance payout vector size overflow"))?;
        reader.skip(
            size,
            format!("expected {insurance_count} insurance payouts of 8 bytes each"),
        )?;
    }
    sections.push(SectionTrace {
        name: "laptop",
        start,
        end: reader.position(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::analyze_bytes;

    #[test]
    fn analyze_bytes_rejects_short_input_without_panicking() {
        let error = analyze_bytes(Path::new("short.sav"), &[0; 10])
            .unwrap_err()
            .to_string();
        assert!(error.contains("432-byte portable save header"));
    }

    #[test]
    fn analyze_bytes_rejects_unsupported_version_at_header() {
        let mut bytes = vec![0; 432];
        bytes[..4].copy_from_slice(&101u32.to_le_bytes());
        let error = analyze_bytes(Path::new("old.sav"), &bytes)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unsupported JA2 save version 101"));
        assert!(error.contains("102, 103"));
    }
}

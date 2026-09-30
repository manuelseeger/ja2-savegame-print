use std::path::Path;

use super::{reader::Reader, world_items::parse_sector_items};
use crate::sector::Sector;

fn u32_bytes(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend(value.to_le_bytes());
}

fn file(bytes: &mut Vec<u8>, payload: &[u8]) {
    u32_bytes(bytes, payload.len() as u32);
    bytes.extend(payload);
}

fn item(exists: bool, index: u16, quantity: u8, visible: i8) -> [u8; 52] {
    let mut record = [0; 52];
    record[0] = u8::from(exists);
    record[2..4].copy_from_slice(&1234i16.to_le_bytes());
    record[4] = 1;
    record[8..10].copy_from_slice(&index.to_le_bytes());
    record[10] = quantity;
    record[44..46].copy_from_slice(&0x40u16.to_le_bytes());
    record[47] = visible as u8;
    record
}

fn item_file(records: &[[u8; 52]]) -> Vec<u8> {
    let mut bytes = Vec::new();
    u32_bytes(&mut bytes, records.len() as u32);
    for record in records {
        bytes.extend(record);
    }
    bytes
}

/// Build the exact post-soldier section sequence, with all variable branches.
/// Surface A1 has all temp types. A2 has an empty item file; A3 has no item file.
/// Underground list order is B4-2 before A1-1, deliberately not grid order.
fn suffix(payload: &[u8]) -> (Vec<u8>, usize) {
    let mut bytes = Vec::new();
    for embedded in [vec![1, 2, 3], vec![], vec![4, 5]] {
        file(&mut bytes, &embedded);
    }
    u32_bytes(&mut bytes, 2); // emails
    for subject in [vec![], vec![0x55; 14]] {
        file(&mut bytes, &subject);
        bytes.extend([0; 44]);
    }
    bytes.extend(vec![0; 18 * 18 * 41]);
    for index in 0..256 {
        let flags = match index {
            0 => 0xff30_0000,
            1 => 0x1000_0000,
            _ => 0,
        };
        u32_bytes(&mut bytes, flags);
        bytes.extend([0; 112]);
    }
    bytes.extend(vec![0; 18 * 18 + 1]);
    u32_bytes(&mut bytes, 2); // underground nodes
    for (x, y, z) in [(4, 2, 2), (1, 1, 1)] {
        u32_bytes(&mut bytes, 0x1000_0000);
        bytes.extend([x, y, z]);
        bytes.extend([0; 65]);
    }
    bytes.extend(vec![0; 20 * 6 * 12 + 20]);
    u32_bytes(&mut bytes, 3); // movement groups
    let mut player = [0; 84];
    player[1] = 1;
    player[5] = 2;
    bytes.extend(player);
    u32_bytes(&mut bytes, 2);
    u32_bytes(&mut bytes, 57);
    u32_bytes(&mut bytes, 58);
    u32_bytes(&mut bytes, 1);
    bytes.extend([0; 8]); // waypoint
    player[5] = 0;
    bytes.extend(player); // empty player group has no player list
    u32_bytes(&mut bytes, 0); // waypoints
    bytes.extend([0; 84]); // enemy
    bytes.extend([0; 29]);
    u32_bytes(&mut bytes, 2);
    bytes.extend([0; 16]);
    bytes.extend([0; 32]); // unique ID mask
    let temp_start = bytes.len();
    file(&mut bytes, payload);
    // All other temp files contain false item signatures. They must be skipped.
    for i in 0..9 {
        let mut decoy = item_file(&[item(true, 305, 8, 1)]);
        decoy.push(i);
        file(&mut bytes, &decoy);
    }
    file(&mut bytes, &item_file(&[]));
    file(&mut bytes, &item_file(&[item(true, 305, 2, -1)]));
    file(&mut bytes, &item_file(&[item(true, 305, 3, 1)]));
    (bytes, temp_start)
}

#[test]
fn saved_ground_counts_include_hidden_stacks_but_not_removed_or_other_items() {
    let payload = item_file(&[
        item(true, 305, 1, 1),
        item(false, 305, 8, 1),
        item(true, 305, 6, -1),
        item(true, 306, 4, 0),
    ]);
    let (mut bytes, temp_start) = suffix(&payload);
    let end = bytes.len();
    bytes.extend([0x77; 23]); // next save section, not map temp data
    let mut reader = Reader::new(&bytes, Path::new("synthetic.sav"));
    let mut traces = Vec::new();
    let sectors = parse_sector_items(&mut reader, &mut traces).unwrap();
    assert_eq!(reader.position(), end);
    assert_eq!(sectors.len(), 258);
    assert_eq!(sectors[0].sector, Sector::new(1, 1, 0));
    assert_eq!(sectors[0].count(305), Some(7));
    assert_eq!(sectors[0].count(306), Some(4));
    assert_eq!(sectors[0].count(999), Some(0));
    let records = sectors[0].items.as_ref().unwrap();
    assert_eq!(records.len(), 4);
    assert!(!records[1].exists);
    assert_eq!(records[2].visible, -1);
    assert_eq!(records[0].grid_no, 1234);
    assert_eq!(records[0].level, 1);
    assert_eq!(records[0].flags, 0x40);
    assert_eq!(sectors[1].count(305), Some(0));
    assert!(sectors[1].items.as_ref().unwrap().is_empty());
    assert_eq!(sectors[2].count(305), None);
    assert_eq!(sectors[256].sector, Sector::new(4, 2, 2));
    assert_eq!(sectors[256].count(305), Some(2));
    assert_eq!(sectors[257].sector, Sector::new(1, 1, 1));
    assert_eq!(sectors[257].count(305), Some(3));
    let trace = traces.last().unwrap();
    assert_eq!(trace.name, "map_temp_files");
    assert_eq!((trace.start, trace.end), (temp_start, end));
}

#[test]
fn truncated_post_soldier_sections_always_return_errors() {
    let (bytes, _) = suffix(&item_file(&[item(true, 305, 1, 1)]));
    // Every prefix truncation must fail, including count fields and skipped files.
    for end in 0..bytes.len() {
        let mut reader = Reader::new(&bytes[..end], Path::new("truncated.sav"));
        reader.set_save_version(102);
        let error = parse_sector_items(&mut reader, &mut Vec::new()).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("truncated.sav"), "{message}");
        assert!(message.contains("version 102"), "{message}");
    }
}

#[test]
fn malformed_item_file_counts_and_lengths_return_errors() {
    for payload in [
        vec![],
        vec![0; 3],
        1u32.to_le_bytes().to_vec(),
        u32::MAX.to_le_bytes().to_vec(),
        vec![0; 5], // trailing byte after zero-count file
        {
            let mut bytes = item_file(&[item(true, 305, 1, 1)]);
            bytes.pop();
            bytes
        },
    ] {
        let (bytes, _) = suffix(&payload);
        let mut reader = Reader::new(&bytes, Path::new("bad-items.sav"));
        let error = parse_sector_items(&mut reader, &mut Vec::new())
            .unwrap_err()
            .to_string();
        assert!(error.contains("MapTempFiles"), "{error}");
        assert!(error.contains("item temp file"), "{error}");
    }
}

#[test]
fn analyze_bytes_exposes_known_empty_and_unknown_sector_counts() {
    let fixture = std::fs::read("fixtures/savegames/2026-08-12t12-19-22z-tixa-done.sav").unwrap();
    let original = super::analyze_bytes(Path::new("fixture.sav"), &fixture).unwrap();
    let soldier_end = original
        .sections
        .iter()
        .find(|section| section.name == "soldier_structure")
        .unwrap()
        .end;
    let (tail, _) = suffix(&item_file(&[
        item(true, 305, 5, -1),
        item(false, 305, 8, 1),
    ]));
    let mut bytes = fixture[..soldier_end].to_vec();
    bytes.extend(tail);
    for version in [102u32, 103] {
        bytes[..4].copy_from_slice(&version.to_le_bytes());
        let analysis = super::analyze_bytes(Path::new("synthetic-full.sav"), &bytes).unwrap();
        assert_eq!(
            analysis.sector_item_count(&Sector::new(1, 1, 0), 305),
            Some(5)
        );
        assert_eq!(
            analysis.sector_item_count(&Sector::new(2, 1, 0), 305),
            Some(0)
        );
        assert_eq!(analysis.sector_item_count(&Sector::new(3, 1, 0), 305), None);
        assert_eq!(analysis.sector_item_count(&Sector::new(1, 1, 3), 305), None);

        let rod = crate::item::lookup("rod").unwrap();
        let mut output = Vec::new();
        crate::output::write_output(
            &analysis,
            &crate::output::OutputOptions {
                json: true,
                pretty: false,
                all_profiles: false,
                list_npcs: false,
                item: Some(&rod),
                items: None,
                include: &[],
                exclude: &[],
            },
            &mut output,
        )
        .unwrap();
        let document: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let locations = document["item"]["locations"].as_array().unwrap();
        let found = |name: &str| {
            locations
                .iter()
                .find(|location| location["sector"]["name"] == name)
                .unwrap()["found"]
                .clone()
        };
        assert_eq!(found("A1"), 5);
        assert_eq!(found("B4-2"), 2);
        assert_eq!(found("A1-1"), 3);
        assert!(found("C5").is_null());
    }
}

#[test]
fn fixture_traverses_all_soldier_slots_to_saved_world_items() {
    // Overhead_Types.h defines 148 regular + 8 planning soldier slots.
    // Stopping at 148 misreads the embedded files in this real save.
    let fixture = std::fs::read("fixtures/savegames/2026-08-12t12-19-22z-tixa-done.sav").unwrap();
    let analysis = super::analyze_bytes(Path::new("fixture.sav"), &fixture).unwrap();
    assert_eq!(analysis.sector_items.len(), 292);
    assert_eq!(
        analysis
            .sector_items
            .iter()
            .filter(|sector| sector.items.is_some())
            .count(),
        33
    );
    let map_temp = analysis
        .sections
        .iter()
        .find(|section| section.name == "map_temp_files")
        .unwrap();
    assert_eq!(map_temp.size(), 1_647_060);
}

#[test]
fn excessive_embedded_file_length_returns_contextual_error() {
    let mut bytes = Vec::new();
    u32_bytes(&mut bytes, u32::MAX);
    let mut reader = Reader::new(&bytes, Path::new("bad-size.sav"));
    let error = parse_sector_items(&mut reader, &mut Vec::new())
        .unwrap_err()
        .to_string();
    assert!(error.contains("finances_file"));
    assert!(error.contains("embedded file payload"));
}

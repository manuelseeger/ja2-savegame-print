use std::{fs, path::Path};

use ja2_savegame::{analyze_bytes, analyze_file, profile::NUM_PROFILES};

const FIXTURE: &str = "fixtures/savegames/2026-08-12t12-19-22z-tixa-done.sav";

#[test]
fn supported_fixture_parses_all_profiles_structurally() {
    let analysis = analyze_file(Path::new(FIXTURE)).expect("fixture should parse");

    assert_eq!(analysis.header.save_version.get(), 102);
    assert_eq!(analysis.profiles.len(), NUM_PROFILES);
    assert!(analysis.header.day > 0);
    for name in ["Hamous", "Skyrider", "Devin", "Carmen", "Micky", "Gabby"] {
        assert!(
            analysis
                .profiles
                .iter()
                .any(|profile| profile.matches_name(name)),
            "missing stock profile {name}"
        );
    }
}

#[test]
fn version_103_uses_the_reviewed_identical_pre_profile_layout() {
    let mut bytes = fs::read(FIXTURE).expect("fixture should be readable");
    bytes[..4].copy_from_slice(&103u32.to_le_bytes());

    let analysis = analyze_bytes(Path::new("version-103.sav"), &bytes)
        .expect("version 103 pre-profile layout should parse");

    assert_eq!(analysis.header.save_version.get(), 103);
    assert_eq!(analysis.profiles.len(), NUM_PROFILES);
}

#[test]
fn dynamic_laptop_vectors_are_skipped_structurally() {
    let mut bytes = fs::read(FIXTURE).expect("fixture should be readable");
    let original = analyze_bytes(Path::new("original.sav"), &bytes).unwrap();
    let laptop_start = original
        .sections
        .iter()
        .find(|section| section.name == "laptop")
        .unwrap()
        .start;
    let profile_start = original
        .sections
        .iter()
        .find(|section| section.name == "merc_profiles")
        .unwrap()
        .start;

    bytes[laptop_start + 7276] = 2;
    bytes[laptop_start + 7277] = 1;
    bytes[laptop_start + 7284] = 3;
    bytes[laptop_start + 7285] = 1;
    let payload_size = 2 * 84 + 3 * 8;
    bytes.splice(profile_start..profile_start, vec![0; payload_size]);

    let analysis = analyze_bytes(Path::new("dynamic-laptop.sav"), &bytes)
        .expect("dynamic vectors should be skipped");

    let shifted_profile_start = analysis
        .sections
        .iter()
        .find(|section| section.name == "merc_profiles")
        .unwrap()
        .start;
    assert_eq!(shifted_profile_start, profile_start + payload_size);
    assert_eq!(analysis.profiles.len(), NUM_PROFILES);
}

#[test]
fn plausible_legacy_linux_header_is_rejected_explicitly() {
    let mut bytes = vec![0; 688];
    bytes[..4].copy_from_slice(&102u32.to_le_bytes());
    bytes[536..540].copy_from_slice(&1u32.to_le_bytes());
    bytes[542..544].copy_from_slice(&1i16.to_le_bytes());
    bytes[544..546].copy_from_slice(&1i16.to_le_bytes());
    bytes[546] = 0;
    bytes[548..552].copy_from_slice(&100i32.to_le_bytes());
    bytes[561] = 1;

    let error = analyze_bytes(Path::new("legacy.sav"), &bytes)
        .unwrap_err()
        .to_string();

    assert_eq!(error, "legacy Stracciatella Linux saves are not supported");
}

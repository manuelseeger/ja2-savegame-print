use std::process::Command;

use ja2_savegame::save::STRACCIATELLA_SOURCE_COMMIT;

const FIXTURE: &str = "fixtures/savegames/2026-08-12t12-19-22z-tixa-done.sav";
const ISSUE_3_FIXTURE: &str = "fixtures/issues/3/Auto01.sav";

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ja2-savegame"))
}

#[test]
fn json_output_is_valid_and_all_profiles_returns_every_profile() {
    let output = binary()
        .args([FIXTURE, "--json", "npc", "--all-profiles"])
        .output()
        .expect("CLI should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(document["header"]["save_version"], 102);
    assert_eq!(document["npcs"].as_array().unwrap().len(), 170);
}

#[test]
fn exclusion_filter_takes_precedence_over_include_filter() {
    let output = binary()
        .args([
            FIXTURE,
            "--json",
            "npc",
            "--all-profiles",
            "-i",
            "Hamous",
            "--exclude",
            "HAMOUS",
        ])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(document["npcs"].as_array().unwrap().is_empty());
}

#[test]
fn list_npcs_prints_names_without_locations() {
    let output = binary()
        .args([FIXTURE, "npc", "--list"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.first(), Some(&"Nickname   Full name"));
    assert!(lines.contains(&"Hamous     Hamous"));
    assert!(lines.contains(&"RPC65      Eskimo - Removed"));
    let nicknames = lines
        .iter()
        .skip(1)
        .map(|line| line.split_whitespace().next().unwrap().to_lowercase())
        .collect::<Vec<_>>();
    assert!(nicknames.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(!stdout.contains("("));
    assert!(!stdout.contains("J1-1"));
}

#[test]
fn issue_3_reports_current_npc_and_rpc_sectors() {
    let output = binary()
        .args([
            ISSUE_3_FIXTURE,
            "npc",
            "--include",
            "Hamous",
            "-i",
            "Dynamo",
            "-i",
            "Ira",
            "-i",
            "Devin",
            "-i",
            "Carmen",
            "-i",
            "Micky",
        ])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Ira Smythe            D13      (13,4,0)",
        "Devin Connell         G9       (9,7,0)",
        "Hamous                J9       (9,10,0)",
        "Greg \"Dynamo\" Duncan  J9-1     (9,10,1)",
        "Carmen Dancio         C5       (5,3,0)",
        "Micky O'Brien         H2       (2,8,0)",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?} in\n{stdout}"
        );
    }
}

#[test]
fn aluminum_rod_locations_are_available_as_text_and_json() {
    let output = binary()
        .args([FIXTURE, "items", "-i", "aluminum-rod"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Aluminum rod (item 305)"));
    assert!(stdout.contains("Sector  Count  Absent chance  Found"));
    assert!(!stdout.contains("unknown"));
    assert!(stdout
        .lines()
        .any(|line| line.starts_with("C5 ") && line.contains("40%")));
    assert!(stdout
        .lines()
        .any(|line| line.starts_with("H3-1 ") && line.contains("30%")));
    assert!(stdout
        .lines()
        .any(|line| line.starts_with("O4 ") && line.contains("30%")));

    let output = binary()
        .args(["--json", FIXTURE, "items", "--include", "aluminum rod"])
        .output()
        .expect("CLI should run");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["item"]["item_index"], 305);
    let locations = document["item"]["locations"].as_array().unwrap();
    assert!(locations.len() >= 11);
    assert!(locations
        .iter()
        .all(|location| location.get("found").is_some()));
}

#[test]
fn items_without_include_shows_all_and_rod_is_an_alias() {
    let all = binary()
        .args([FIXTURE, "items"])
        .output()
        .expect("CLI should run");
    let rod = binary()
        .args([FIXTURE, "items", "-i", "rod"])
        .output()
        .expect("CLI should run");
    assert!(all.status.success());
    assert!(rod.status.success());
    let all_text = String::from_utf8_lossy(&all.stdout);
    assert!(all_text.starts_with(String::from_utf8_lossy(&rod.stdout).as_ref()));
    assert!(all_text.contains("Spring (item 306)"));

    let all_json = binary()
        .args([FIXTURE, "items", "--json"])
        .output()
        .expect("CLI should run");
    assert!(all_json.status.success());
    let document: serde_json::Value = serde_json::from_slice(&all_json.stdout).unwrap();
    assert_eq!(document["items"].as_array().unwrap().len(), 7);
    for (index, id) in [305, 306, 315, 308, 318, 319, 316].iter().enumerate() {
        assert_eq!(document["items"][index]["item_index"], *id);
    }
}

#[test]
fn spring_locations_are_available_as_text_and_json() {
    let output = binary()
        .args([FIXTURE, "items", "--include", "spring"])
        .output()
        .expect("CLI should run");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.starts_with("Spring (item 306)"));
    assert!(text
        .lines()
        .any(|line| line.starts_with("G1 ") && line.contains("20%")));
    assert!(text
        .lines()
        .any(|line| line.starts_with("I14-1 ") && line.contains("40%")));
    assert!(text
        .lines()
        .any(|line| line.starts_with("O4 ") && line.contains("40%")));

    let output = binary()
        .args([FIXTURE, "items", "-i", "SPRING", "--json"])
        .output()
        .expect("CLI should run");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["item"]["internal_name"], "SPRING");
    let locations = document["item"]["locations"].as_array().unwrap();
    assert!(locations.len() >= 10);
    assert!(locations
        .iter()
        .all(|location| location.get("found").is_some()));
}

#[test]
fn components_are_available_as_text_and_json() {
    for (alias, name, internal_name, id) in [
        ("lame-boy", "Lame boy", "LAME_BOY", 315),
        ("STEEL_ROD", "Steel rod", "STEEL_ROD", 308),
        ("Fumble pak", "Fumble pak", "FUMBLE_PAK", 318),
        ("X-ray bulb", "X-ray bulb", "XRAY_BULB", 319),
        ("copper_wire", "Copper wire", "COPPER_WIRE", 316),
    ] {
        let output = binary()
            .args([FIXTURE, "items", "-i", alias])
            .output()
            .unwrap();
        assert!(output.status.success(), "{alias}");
        assert!(String::from_utf8_lossy(&output.stdout).starts_with(&format!("{name} (item {id})")));
        let output = binary()
            .args([FIXTURE, "items", "-i", internal_name, "--json"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{internal_name}");
        let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(document["item"]["item_index"], id);
        assert_eq!(document["item"]["internal_name"], internal_name);
        assert!(document["item"]["locations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|location| location.get("found").is_some()));
    }
}

#[test]
fn command_options_are_scoped_and_general_options_work_after_commands() {
    let output = binary()
        .args([FIXTURE, "npc", "--list", "--json", "--pretty"])
        .output()
        .expect("CLI should run");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(document["npcs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|npc| npc["nickname"] == "Hamous"));

    for args in [
        vec![FIXTURE, "items", "--all-profiles", "-i", "aluminum-rod"],
        vec![FIXTURE, "npc", "--item", "aluminum-rod"],
    ] {
        let output = binary().args(args).output().expect("CLI should run");
        assert_eq!(output.status.code(), Some(2));
    }
}

#[test]
fn missing_command_is_an_error() {
    let output = binary().arg(FIXTURE).output().expect("CLI should run");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("a command is required"));
}

#[test]
fn multiple_input_paths_are_a_usage_error() {
    let output = binary()
        .args([FIXTURE, "npc", FIXTURE])
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
}

#[test]
fn source_version_prints_pinned_commit_without_a_save_file() {
    let output = binary()
        .arg("--source-version")
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        STRACCIATELLA_SOURCE_COMMIT
    );
}

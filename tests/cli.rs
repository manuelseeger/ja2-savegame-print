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
        .args([FIXTURE, "--json", "--all-profiles"])
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
            "--all-profiles",
            "--npc",
            "Hamous",
            "--exclude-npc",
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
        .args([FIXTURE, "--list-npcs"])
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
            "--npc",
            "Hamous",
            "--npc",
            "Dynamo",
            "--npc",
            "Ira",
            "--npc",
            "Devin",
            "--npc",
            "Carmen",
            "--npc",
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
        .args([FIXTURE, "--item", "aluminum-rod"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Aluminum rod (item 305)"));
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
        .args([FIXTURE, "--item", "aluminum rod", "--json"])
        .output()
        .expect("CLI should run");
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["item"]["item_index"], 305);
    assert_eq!(document["item"]["locations"].as_array().unwrap().len(), 11);
}

#[test]
fn multiple_input_paths_are_a_usage_error() {
    let output = binary()
        .args([FIXTURE, FIXTURE])
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

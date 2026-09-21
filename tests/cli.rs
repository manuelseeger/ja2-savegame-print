use std::process::Command;

use ja2_savegame::save::STRACCIATELLA_SOURCE_COMMIT;

const FIXTURE: &str = "fixtures/savegames/2026-08-12t12-19-22z-tixa-done.sav";

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
    assert!(lines.iter().any(|line| *line == "Hamous     Hamous"));
    assert!(lines
        .iter()
        .any(|line| *line == "RPC65      Eskimo - Removed"));
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

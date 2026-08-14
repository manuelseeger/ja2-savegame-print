use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(name = "ja2-savegame", version, about)]
pub struct Cli {
    /// Print the pinned Stracciatella source commit and exit.
    #[arg(long, global = true)]
    pub source_version: bool,

    /// Show diagnostic details (-vv shows more).
    #[arg(short = 'v', action = ArgAction::Count)]
    pub verbose: u8,

    /// Save file to inspect.
    #[arg(value_name = "FILE", required_unless_present = "source_version")]
    pub file: Option<PathBuf>,

    /// Output as JSON instead of plain text.
    #[arg(long)]
    pub json: bool,

    /// Pretty-print JSON (requires --json).
    #[arg(long, requires = "json")]
    pub pretty: bool,

    /// Include every character, even those with no known name or location.
    #[arg(long)]
    pub all_profiles: bool,

    /// Show a character by name (repeatable; --npc is an alias).
    #[arg(long = "include-npc", visible_alias = "npc", value_name = "NAME")]
    pub include_npc: Vec<String>,

    /// Exclude a character by name (repeatable; takes precedence).
    #[arg(long, value_name = "NAME")]
    pub exclude_npc: Vec<String>,
}

use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ja2-savegame", version, about)]
pub struct Cli {
    /// Print the pinned Stracciatella source commit and exit.
    #[arg(long, global = true)]
    pub source_version: bool,

    /// Show parsed section offsets (-vv includes section sizes).
    #[arg(short = 'v', action = ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Inspect one Stracciatella save file.
    Inspect(InspectArgs),
}

#[derive(Debug, Args)]
pub struct InspectArgs {
    /// Exactly one .sav file to inspect.
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Emit stable JSON instead of plaintext.
    #[arg(long)]
    pub json: bool,

    /// Pretty-print JSON (requires --json).
    #[arg(long, requires = "json")]
    pub pretty: bool,

    /// Include all 170 profiles, including unmapped and unplaced profiles.
    #[arg(long)]
    pub all_profiles: bool,

    /// Include a name (repeatable; --npc is an alias).
    #[arg(long = "include-npc", visible_alias = "npc", value_name = "NAME")]
    pub include_npc: Vec<String>,

    /// Exclude a name (repeatable; exclusions take precedence).
    #[arg(long, value_name = "NAME")]
    pub exclude_npc: Vec<String>,
}

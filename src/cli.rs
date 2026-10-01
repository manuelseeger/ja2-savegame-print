use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ja2-savegame", version, about)]
pub struct Cli {
    /// Print the pinned Stracciatella source commit and exit.
    #[arg(long, global = true)]
    pub source_version: bool,

    /// Show diagnostic details (-vv shows more).
    #[arg(short = 'v', action = ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Save file to inspect.
    #[arg(value_name = "FILE", required_unless_present = "source_version")]
    pub file: Option<PathBuf>,

    /// Output as JSON instead of plain text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Pretty-print JSON (requires --json).
    #[arg(long, requires = "json", global = true)]
    pub pretty: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show NPC locations or names.
    Npc {
        /// Include every character, even those with no known name or location.
        #[arg(long)]
        all_profiles: bool,

        /// Print NPC nicknames and full names without locations.
        #[arg(long = "list")]
        list_npcs: bool,

        /// Show a character by name (repeatable).
        #[arg(short = 'i', long = "include", value_name = "NAME")]
        include: Vec<String>,

        /// Exclude a character by name (repeatable; takes precedence).
        #[arg(long, value_name = "NAME")]
        exclude: Vec<String>,
    },
    /// Show known map placements for an item.
    Items {
        /// Item to show: aluminum-rod (rod), spring, lame-boy, steel-rod,
        /// fumble-pak, x-ray-bulb, or copper-wire. Omit to show all items.
        #[arg(short = 'i', long = "include", value_name = "ITEM")]
        include: Option<String>,
    },
}

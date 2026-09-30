use std::{io, process::ExitCode};

use clap::Parser;
use ja2_savegame::{
    analyze_file,
    cli::{Cli, Command},
    output::{write_output, OutputOptions},
    save::STRACCIATELLA_SOURCE_COMMIT,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), io::Error> {
    let cli = Cli::parse();
    if cli.source_version {
        println!("{STRACCIATELLA_SOURCE_COMMIT}");
        return Ok(());
    }

    let file = cli
        .file
        .as_deref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "a save file is required"))?;
    let command = cli.command.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "a command is required: npc or items",
        )
    })?;
    let analysis = analyze_file(file).map_err(io::Error::other)?;
    let item = match &command {
        Command::Items {
            include: Some(include),
        } => Some(ja2_savegame::item::lookup(include).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown item {include:?}; supported items: aluminum-rod (rod), spring"),
            )
        })?),
        _ => None,
    };
    let items = match &command {
        Command::Items { include: None } => Some(ja2_savegame::item::all()),
        _ => None,
    };
    if cli.verbose > 0 {
        for section in &analysis.sections {
            if cli.verbose > 1 {
                eprintln!(
                    "0x{:08X}..0x{:08X} {:<20} ({} bytes)",
                    section.start,
                    section.end,
                    section.name,
                    section.size()
                );
            } else {
                eprintln!("0x{:08X} {}", section.start, section.name);
            }
        }
    }
    let (all_profiles, list_npcs, include, exclude) = match &command {
        Command::Npc {
            all_profiles,
            list_npcs,
            include,
            exclude,
        } => (
            *all_profiles,
            *list_npcs,
            include.as_slice(),
            exclude.as_slice(),
        ),
        Command::Items { .. } => (false, false, &[][..], &[][..]),
    };
    write_output(
        &analysis,
        &OutputOptions {
            json: cli.json,
            pretty: cli.pretty,
            all_profiles,
            list_npcs,
            item: item.as_ref(),
            items: items.as_deref(),
            include,
            exclude,
        },
        io::stdout().lock(),
    )
}

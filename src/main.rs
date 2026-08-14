use std::{io, process::ExitCode};

use clap::Parser;
use ja2_savegame::{
    analyze_file,
    cli::Cli,
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
    let analysis = analyze_file(file).map_err(io::Error::other)?;
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
    write_output(
        &analysis,
        &OutputOptions {
            json: cli.json,
            pretty: cli.pretty,
            all_profiles: cli.all_profiles,
            include: &cli.include_npc,
            exclude: &cli.exclude_npc,
        },
        io::stdout().lock(),
    )
}

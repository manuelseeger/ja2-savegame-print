use std::io::{self, Write};

use serde::Serialize;

use crate::{profile::MercProfile, save::SaveAnalysis};

pub struct OutputOptions<'a> {
    pub json: bool,
    pub pretty: bool,
    pub all_profiles: bool,
    pub include: &'a [String],
    pub exclude: &'a [String],
}

pub fn selected_profiles<'a>(
    analysis: &'a SaveAnalysis,
    options: &OutputOptions<'_>,
) -> Vec<&'a MercProfile> {
    analysis
        .profiles
        .iter()
        .filter(|profile| {
            options.all_profiles
                || (profile.is_stock_npc_or_rpc() && profile.has_meaningful_location())
        })
        .filter(|profile| {
            options.include.is_empty()
                || options
                    .include
                    .iter()
                    .any(|name| profile.matches_name(name))
        })
        .filter(|profile| {
            !options
                .exclude
                .iter()
                .any(|name| profile.matches_name(name))
        })
        .collect()
}

pub fn write_output(
    analysis: &SaveAnalysis,
    options: &OutputOptions<'_>,
    mut output: impl Write,
) -> Result<(), io::Error> {
    let profiles = selected_profiles(analysis, options);
    if options.json {
        let document = JsonDocument {
            file: &analysis.file,
            header: &analysis.header,
            npcs: profiles,
        };
        if options.pretty {
            serde_json::to_writer_pretty(&mut output, &document)?;
        } else {
            serde_json::to_writer(&mut output, &document)?;
        }
        writeln!(output)?;
    } else {
        write_text(analysis, &profiles, &mut output)?;
    }
    Ok(())
}

#[derive(Serialize)]
struct JsonDocument<'a> {
    file: &'a str,
    header: &'a crate::save::SaveHeader,
    npcs: Vec<&'a MercProfile>,
}

fn write_text(
    analysis: &SaveAnalysis,
    profiles: &[&MercProfile],
    mut output: impl Write,
) -> Result<(), io::Error> {
    writeln!(output, "{}\n", analysis.file)?;
    writeln!(output, "Save")?;
    writeln!(output, "  format version: {}", analysis.header.save_version)?;
    writeln!(output, "  game version:   {}", analysis.header.game_version)?;
    writeln!(
        output,
        "  time:           Day {} {:02}:{:02}",
        analysis.header.day, analysis.header.hour, analysis.header.minute
    )?;
    writeln!(
        output,
        "  player sector:  {}",
        sector_display(&analysis.header.sector)
    )?;
    writeln!(
        output,
        "  world loaded:   {}\n",
        analysis.header.world_loaded
    )?;
    writeln!(output, "NPCs")?;
    if profiles.is_empty() {
        writeln!(output, "  (none)")?;
        return Ok(());
    }
    let width = profiles
        .iter()
        .map(|profile| profile.display_name().chars().count())
        .max()
        .unwrap_or(1)
        .max(4);
    for profile in profiles {
        let state = if matches!(
            profile.location_state,
            crate::profile::LocationState::Placed
        ) {
            String::new()
        } else {
            format!("  [{}]", state_label(profile.location_state))
        };
        writeln!(
            output,
            "  {:width$}  {:<8} ({},{},{}){}",
            profile.display_name(),
            sector_display(&profile.sector),
            profile.sector.x,
            profile.sector.y,
            profile.sector.z,
            state,
            width = width
        )?;
    }
    Ok(())
}

fn state_label(state: crate::profile::LocationState) -> &'static str {
    use crate::profile::LocationState;
    match state {
        LocationState::Placed => "placed",
        LocationState::NotCurrentlyPlaced => "not currently placed",
        LocationState::Dead => "dead",
        LocationState::Unavailable => "unavailable",
        LocationState::Recruited => "recruited",
        LocationState::Unknown => "unknown",
    }
}

fn sector_display(sector: &crate::sector::Sector) -> String {
    sector
        .name
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "N/A".to_owned())
}

#[cfg(test)]
mod tests {
    use crate::sector::Sector;

    use super::sector_display;

    #[test]
    fn sector_display_does_not_turn_invalid_coordinates_into_a_sector() {
        assert_eq!(sector_display(&Sector::new(0, 0, -1)), "N/A");
    }
}

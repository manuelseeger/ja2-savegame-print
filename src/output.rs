use std::io::{self, Write};

use serde::Serialize;

use crate::{item::Item, profile::MercProfile, save::SaveAnalysis};

pub struct OutputOptions<'a> {
    pub json: bool,
    pub pretty: bool,
    pub all_profiles: bool,
    pub list_npcs: bool,
    pub item: Option<&'a Item>,
    pub items: Option<&'a [Item]>,
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
                || (profile.is_stock_npc_or_rpc()
                    && (options.list_npcs || profile.has_meaningful_location()))
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
    if let Some(items) = options.items {
        if options.json {
            let document = ItemsDocument {
                file: &analysis.file,
                items,
            };
            if options.pretty {
                serde_json::to_writer_pretty(&mut output, &document)?;
            } else {
                serde_json::to_writer(&mut output, &document)?;
            }
            writeln!(output)?;
        } else {
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    writeln!(output)?;
                }
                write_item(item, &mut output)?;
            }
        }
        return Ok(());
    }
    if let Some(item) = options.item {
        if options.json {
            let document = ItemDocument {
                file: &analysis.file,
                item,
            };
            if options.pretty {
                serde_json::to_writer_pretty(&mut output, &document)?;
            } else {
                serde_json::to_writer(&mut output, &document)?;
            }
            writeln!(output)?;
        } else {
            write_item(item, &mut output)?;
        }
        return Ok(());
    }
    let profiles = selected_profiles(analysis, options);
    if options.json {
        if options.list_npcs {
            let entries = sorted_name_list_entries(&profiles);
            let document = JsonNameListDocument {
                file: &analysis.file,
                header: &analysis.header,
                npcs: entries
                    .iter()
                    .map(|entry| JsonNameListEntry {
                        nickname: entry.nickname,
                        full_name: entry.full_name,
                    })
                    .collect(),
            };
            if options.pretty {
                serde_json::to_writer_pretty(&mut output, &document)?;
            } else {
                serde_json::to_writer(&mut output, &document)?;
            }
        } else {
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
        }
        writeln!(output)?;
    } else if options.list_npcs {
        write_name_list(&profiles, &mut output)?;
    } else {
        write_text(analysis, &profiles, &mut output)?;
    }
    Ok(())
}

#[derive(Serialize)]
struct ItemDocument<'a> {
    file: &'a str,
    item: &'a Item,
}

#[derive(Serialize)]
struct ItemsDocument<'a> {
    file: &'a str,
    items: &'a [Item],
}

#[derive(Serialize)]
struct JsonDocument<'a> {
    file: &'a str,
    header: &'a crate::save::SaveHeader,
    npcs: Vec<&'a MercProfile>,
}

#[derive(Serialize)]
struct JsonNameListDocument<'a> {
    file: &'a str,
    header: &'a crate::save::SaveHeader,
    npcs: Vec<JsonNameListEntry<'a>>,
}

#[derive(Serialize)]
struct JsonNameListEntry<'a> {
    nickname: &'a str,
    full_name: &'a str,
}

struct NameListEntry<'a> {
    nickname: &'a str,
    full_name: &'a str,
}

fn sorted_name_list_entries<'a>(profiles: &'a [&'a MercProfile]) -> Vec<NameListEntry<'a>> {
    let mut entries = profiles
        .iter()
        .map(|profile| NameListEntry {
            nickname: profile
                .nickname
                .as_deref()
                .unwrap_or_else(|| profile.display_name()),
            full_name: if profile.name.is_empty() {
                profile.display_name()
            } else {
                &profile.name
            },
        })
        .collect::<Vec<_>>();
    entries.sort_by_cached_key(|entry| {
        (
            entry.nickname.to_lowercase(),
            entry.full_name.to_lowercase(),
        )
    });
    entries
}

fn write_item(item: &Item, mut output: impl Write) -> Result<(), io::Error> {
    writeln!(output, "{} (item {})", item.name, item.item_index)?;
    writeln!(output, "Sector  Count  Absent chance")?;
    for location in &item.locations {
        let sector = location
            .sector
            .name
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "N/A".to_owned());
        writeln!(
            output,
            "{:<6}  {:>5}  {:>13}%",
            sector, location.count, location.absent_chance_percent
        )?;
    }
    Ok(())
}

fn write_name_list(profiles: &[&MercProfile], mut output: impl Write) -> Result<(), io::Error> {
    let entries = sorted_name_list_entries(profiles);
    let nickname_width = entries
        .iter()
        .map(|entry| entry.nickname.chars().count())
        .max()
        .unwrap_or(0)
        .max("Nickname".len());
    writeln!(output, "{:<nickname_width$}  Full name", "Nickname")?;
    for entry in entries {
        writeln!(
            output,
            "{:<nickname_width$}  {}",
            entry.nickname, entry.full_name
        )?;
    }
    Ok(())
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

use serde::Serialize;

use crate::sector::Sector;

#[derive(Debug, Clone, Serialize)]
pub struct ItemLocation {
    pub sector: Sector,
    pub count: u8,
    /// Probability that this map placement is absent when the sector is first entered.
    pub absent_chance_percent: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub name: &'static str,
    pub internal_name: &'static str,
    pub item_index: u16,
    pub locations: Vec<ItemLocation>,
}

pub fn all() -> Vec<Item> {
    vec![
        aluminum_rod(),
        spring(),
        lame_boy(),
        steel_rod(),
        fumble_pak(),
        xray_bulb(),
        copper_wire(),
    ]
}

pub fn lookup(name: &str) -> Option<Item> {
    let normalized = name
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_' && *c != '-')
        .collect::<String>()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "aluminumrod" | "aluminiumrod" | "rod" => Some(aluminum_rod()),
        "spring" => Some(spring()),
        "lameboy" => Some(lame_boy()),
        "steelrod" => Some(steel_rod()),
        "fumblepak" => Some(fumble_pak()),
        "xraybulb" => Some(xray_bulb()),
        "copperwire" => Some(copper_wire()),
        _ => None,
    }
}

// Fixed placements in the original game's Data/Maps.slf. Counts combine map
// records with the same sector and absent chance; each record rolls separately.
fn aluminum_rod() -> Item {
    let placements = [
        (5, 3, 0, 1, 40),   // C5
        (2, 7, 0, 2, 30),   // G2
        (1, 8, 0, 2, 40),   // H1
        (2, 8, 0, 4, 30),   // H2
        (3, 8, 0, 1, 40),   // H3
        (3, 8, 1, 1, 30),   // H3 basement
        (8, 8, 0, 2, 50),   // H8
        (4, 11, 0, 1, 30),  // K4
        (11, 12, 0, 1, 40), // L11
        (7, 14, 0, 1, 15),  // N7
        (4, 15, 0, 2, 30),  // O4
    ];
    item("Aluminum rod", "ALUMINUM_ROD", 305, &placements)
}

fn spring() -> Item {
    let placements = [
        (1, 7, 0, 1, 40),   // G1
        (1, 7, 0, 1, 20),   // G1 (separate absent chance)
        (2, 7, 0, 1, 30),   // G2
        (3, 8, 0, 2, 40),   // H3
        (6, 9, 0, 3, 30),   // I6 (Estoni)
        (14, 9, 1, 1, 40),  // I14 basement
        (14, 10, 0, 1, 30), // J14
        (10, 12, 0, 2, 30), // L10
        (7, 14, 0, 1, 15),  // N7
        (4, 15, 0, 2, 40),  // O4
    ];
    item("Spring", "SPRING", 306, &placements)
}

fn lame_boy() -> Item {
    // No fixed placement in Maps.slf, including alternate maps.
    item("Lame boy", "LAME_BOY", 315, &[])
}

fn steel_rod() -> Item {
    let placements = [
        (1, 7, 0, 1, 30),   // G1
        (2, 7, 0, 1, 50),   // G2
        (1, 8, 0, 1, 40),   // H1
        (3, 8, 0, 1, 40),   // H3
        (6, 9, 0, 1, 20),   // I6
        (6, 9, 0, 1, 30),   // I6 (separate absent chance)
        (14, 9, 0, 1, 40),  // I14
        (9, 10, 1, 1, 20),  // J9 basement, Sci-Fi off (J9_B1_A.DAT)
        (10, 12, 0, 1, 40), // L10
        (11, 12, 0, 1, 40), // L11
    ];
    item("Steel rod", "STEEL_ROD", 308, &placements)
}

fn fumble_pak() -> Item {
    item("Fumble pak", "FUMBLE_PAK", 318, &[(11, 12, 0, 1, 30)]) // L11
}

fn xray_bulb() -> Item {
    let placements = [
        (8, 6, 0, 3, 30),  // F8
        (4, 15, 0, 2, 40), // O4
    ];
    item("X-ray bulb", "XRAY_BULB", 319, &placements)
}

fn copper_wire() -> Item {
    let placements = [
        (2, 2, 0, 1, 30),   // B2
        (6, 3, 0, 1, 30),   // C6
        (8, 6, 0, 1, 10),   // F8
        (8, 7, 0, 1, 50),   // G8
        (9, 10, 1, 1, 20),  // J9 basement, Sci-Fi off (J9_B1_A.DAT)
        (10, 12, 0, 1, 30), // L10
        (11, 12, 0, 1, 40), // L11
        (4, 15, 0, 2, 40),  // O4
    ];
    item("Copper wire", "COPPER_WIRE", 316, &placements)
}

fn item(
    name: &'static str,
    internal_name: &'static str,
    item_index: u16,
    placements: &[(u16, u16, i8, u8, u8)],
) -> Item {
    Item {
        name,
        internal_name,
        item_index,
        locations: placements
            .iter()
            .map(|&(x, y, z, count, absent_chance_percent)| ItemLocation {
                sector: Sector::new(x, y, z),
                count,
                absent_chance_percent,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::lookup;

    #[test]
    fn aluminum_rod_locations_match_map_placements() {
        let item = lookup("aluminum rod").unwrap();
        assert_eq!(item.item_index, 305);
        assert_eq!(item.locations.len(), 11);
        assert_eq!(
            item.locations[0].sector.name.as_ref().unwrap().to_string(),
            "C5"
        );
        assert_eq!(
            item.locations[5].sector.name.as_ref().unwrap().to_string(),
            "H3-1"
        );
        assert_eq!(item.locations[10].count, 2);
    }

    #[test]
    fn item_names_are_case_and_separator_insensitive() {
        assert!(lookup("ALUMINUM_ROD").is_some());
        assert_eq!(lookup(" ROD ").unwrap().item_index, 305);
        assert!(lookup("unknown").is_none());
    }

    #[test]
    fn spring_locations_match_original_maps() {
        let spring = lookup("SPRING").unwrap();
        assert_eq!(spring.internal_name, "SPRING");
        assert_eq!(spring.item_index, 306);
        assert_eq!(spring.locations.len(), 10);
        assert_eq!(
            spring.locations[0]
                .sector
                .name
                .as_ref()
                .unwrap()
                .to_string(),
            "G1"
        );
        assert_eq!(spring.locations[0].absent_chance_percent, 40);
        assert_eq!(spring.locations[1].absent_chance_percent, 20);
        assert_eq!(spring.locations[4].count, 3);
        assert_eq!(
            spring.locations[5]
                .sector
                .name
                .as_ref()
                .unwrap()
                .to_string(),
            "I14-1"
        );
        assert_eq!(spring.locations[9].count, 2);
    }

    #[test]
    fn component_locations_match_original_maps() {
        let cases = [
            ("LAME_BOY", 315, vec![]),
            (
                "STEEL_ROD",
                308,
                vec![
                    (1, 7, 0, 1, 30),
                    (2, 7, 0, 1, 50),
                    (1, 8, 0, 1, 40),
                    (3, 8, 0, 1, 40),
                    (6, 9, 0, 1, 20),
                    (6, 9, 0, 1, 30),
                    (14, 9, 0, 1, 40),
                    (9, 10, 1, 1, 20),
                    (10, 12, 0, 1, 40),
                    (11, 12, 0, 1, 40),
                ],
            ),
            ("FUMBLE_PAK", 318, vec![(11, 12, 0, 1, 30)]),
            ("XRAY_BULB", 319, vec![(8, 6, 0, 3, 30), (4, 15, 0, 2, 40)]),
            (
                "COPPER_WIRE",
                316,
                vec![
                    (2, 2, 0, 1, 30),
                    (6, 3, 0, 1, 30),
                    (8, 6, 0, 1, 10),
                    (8, 7, 0, 1, 50),
                    (9, 10, 1, 1, 20),
                    (10, 12, 0, 1, 30),
                    (11, 12, 0, 1, 40),
                    (4, 15, 0, 2, 40),
                ],
            ),
        ];
        for (name, id, expected) in cases {
            let item = lookup(name).unwrap();
            assert_eq!(item.item_index, id);
            assert_eq!(item.internal_name, name);
            let actual = item
                .locations
                .iter()
                .map(|location| {
                    (
                        location.sector.x,
                        location.sector.y,
                        location.sector.z,
                        location.count,
                        location.absent_chance_percent,
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{name}");
        }
    }

    #[test]
    fn component_aliases_ignore_case_and_separators() {
        for (names, id) in [
            (["Lame boy", "LAME_BOY", "lame-boy"], 315),
            (["Steel rod", "STEEL_ROD", "steel-rod"], 308),
            (["Fumble pak", "FUMBLE_PAK", "fumble-pak"], 318),
            (["X-ray bulb", "XRAY_BULB", "x_ray-bulb"], 319),
            (["Copper wire", "COPPER_WIRE", "copper-wire"], 316),
        ] {
            for name in names {
                assert_eq!(lookup(name).unwrap().item_index, id, "{name}");
            }
        }
    }

    #[test]
    fn all_items_contains_every_supported_item() {
        let items = super::all();
        assert_eq!(
            items.iter().map(|item| item.item_index).collect::<Vec<_>>(),
            vec![305, 306, 315, 308, 318, 319, 316]
        );
    }
}

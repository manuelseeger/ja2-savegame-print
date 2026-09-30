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
    vec![aluminum_rod(), spring()]
}

pub fn lookup(name: &str) -> Option<Item> {
    let normalized = name.trim().to_ascii_lowercase().replace([' ', '_'], "-");
    match normalized.as_str() {
        "aluminum-rod" | "aluminium-rod" | "rod" => Some(aluminum_rod()),
        "spring" => Some(spring()),
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
    fn all_items_contains_rod_and_spring() {
        let items = super::all();
        assert_eq!(
            items.iter().map(|item| item.item_index).collect::<Vec<_>>(),
            vec![305, 306]
        );
    }
}

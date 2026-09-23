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

pub fn lookup(name: &str) -> Option<Item> {
    let normalized = name.trim().to_ascii_lowercase().replace([' ', '_'], "-");
    if !["aluminum-rod", "aluminium-rod"].contains(&normalized.as_str()) {
        return None;
    }

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
    Some(Item {
        name: "Aluminum rod",
        internal_name: "ALUMINUM_ROD",
        item_index: 305,
        locations: placements
            .into_iter()
            .map(|(x, y, z, count, absent_chance_percent)| ItemLocation {
                sector: Sector::new(x, y, z),
                count,
                absent_chance_percent,
            })
            .collect(),
    })
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
        assert!(lookup("unknown").is_none());
    }
}

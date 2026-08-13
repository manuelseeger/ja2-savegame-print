use serde::Serialize;

/// A strategic JA2 sector. Coordinates outside the 1..=16 surface grid are
/// retained but have no friendly name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Sector {
    pub x: u16,
    pub y: u16,
    pub z: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<SectorName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SectorName(String);

impl std::fmt::Display for SectorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Sector {
    pub fn new(x: u16, y: u16, z: i8) -> Self {
        let name = Self::format_name(x, y, z).map(SectorName);
        Self { x, y, z, name }
    }

    pub fn is_valid(&self) -> bool {
        self.name.is_some()
    }

    fn format_name(x: u16, y: u16, z: i8) -> Option<String> {
        if !(1..=16).contains(&x) || !(1..=16).contains(&y) || !(0..=3).contains(&z) {
            return None;
        }
        let column = char::from(b'A' + (x as u8 - 1));
        if z == 0 {
            Some(format!("{column}{y}"))
        } else {
            Some(format!("{column}{y}-{z}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sector;

    #[test]
    fn new_surface_sector_formats_ja2_notation() {
        let sector = Sector::new(15, 4, 0);
        assert_eq!(sector.name.unwrap().to_string(), "O4");
    }

    #[test]
    fn new_underground_sector_includes_depth() {
        let sector = Sector::new(1, 9, 2);
        assert_eq!(sector.name.unwrap().to_string(), "A9-2");
    }

    #[test]
    fn new_invalid_sector_preserves_raw_coordinates_without_name() {
        let sector = Sector::new(0, 99, -1);
        assert_eq!((sector.x, sector.y, sector.z), (0, 99, -1));
        assert!(sector.name.is_none());
    }

    #[test]
    fn new_sector_rejects_depth_beyond_ja2_limit() {
        let sector = Sector::new(1, 1, 4);
        assert!(sector.name.is_none());
    }
}

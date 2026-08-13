use serde::Serialize;

use crate::save::ParseError;

/// Pinned Stracciatella format reference used throughout this crate.
pub const STRACCIATELLA_SOURCE_COMMIT: &str = "dcc20b3c24b3e49ccd16e9d4ae87dcd20b9e51ea";
pub const SUPPORTED_VERSION_TEXT: &str = "102, 103";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SupportedSaveVersion(u32);

impl SupportedSaveVersion {
    pub fn new(value: u32) -> Result<Self, ParseError> {
        match value {
            102 | 103 => Ok(Self(value)),
            unsupported => Err(ParseError::UnsupportedVersion(unsupported)),
        }
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for SupportedSaveVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::SupportedSaveVersion;

    #[test]
    fn new_accepts_only_reviewed_versions() {
        assert!(SupportedSaveVersion::new(102).is_ok());
        assert!(SupportedSaveVersion::new(103).is_ok());
        assert!(SupportedSaveVersion::new(101).is_err());
        assert!(SupportedSaveVersion::new(104).is_err());
    }
}

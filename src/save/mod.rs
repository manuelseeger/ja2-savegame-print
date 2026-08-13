mod encryption;
pub mod header;
mod parser;
pub mod reader;
mod version;

use std::path::PathBuf;

pub use header::SaveHeader;
pub use parser::{analyze_bytes, analyze_file, SaveAnalysis, SectionTrace};
pub use version::{SupportedSaveVersion, STRACCIATELLA_SOURCE_COMMIT, SUPPORTED_VERSION_TEXT};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unsupported JA2 save version {0}; supported English portable versions: 102, 103")]
    UnsupportedVersion(u32),

    #[error("legacy Stracciatella Linux saves are not supported")]
    LegacyLinux,

    #[error("could not read {file}: {source}")]
    Io {
        file: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "{file}: offset 0x{offset:X}, {section}: {operation}{version_suffix}",
        version_suffix = save_version.map(|v| format!(" (save version {v})")).unwrap_or_default()
    )]
    Format {
        file: PathBuf,
        offset: usize,
        section: &'static str,
        operation: String,
        save_version: Option<u32>,
    },

    #[error("MercProfiles: profile {profile_id}: {message}")]
    Profile { profile_id: u8, message: String },
}

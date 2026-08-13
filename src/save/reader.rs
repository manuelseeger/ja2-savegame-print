use std::path::{Path, PathBuf};

use crate::save::ParseError;

/// Bounds-checked little-endian reader over one save file.
pub struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    file: PathBuf,
    section: &'static str,
    save_version: Option<u32>,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8], file: &Path) -> Self {
        Self {
            data,
            position: 0,
            file: file.to_path_buf(),
            section: "Header",
            save_version: None,
        }
    }

    pub fn set_section(&mut self, section: &'static str) {
        self.section = section;
    }

    pub fn set_save_version(&mut self, version: u32) {
        self.save_version = Some(version);
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    pub fn seek(
        &mut self,
        position: usize,
        operation: impl Into<String>,
    ) -> Result<(), ParseError> {
        if position > self.data.len() {
            return Err(self.error(operation));
        }
        self.position = position;
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        Ok(self.read_bytes(1, "expected one u8")?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, ParseError> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, ParseError> {
        let bytes = self.read_bytes(2, "expected one little-endian u16")?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_i16_le(&mut self) -> Result<i16, ParseError> {
        Ok(self.read_u16_le()? as i16)
    }

    pub fn read_u32_le(&mut self) -> Result<u32, ParseError> {
        let bytes = self.read_bytes(4, "expected one little-endian u32")?;
        Ok(u32::from_le_bytes(
            bytes.try_into().expect("four-byte slice"),
        ))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, ParseError> {
        Ok(self.read_u32_le()? as i32)
    }

    pub fn read_bytes(
        &mut self,
        count: usize,
        operation: impl Into<String>,
    ) -> Result<&'a [u8], ParseError> {
        let operation = operation.into();
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| self.error(&operation))?;
        if end > self.data.len() {
            return Err(self.error(format!(
                "{operation}; need {count} bytes, only {} remain",
                self.remaining()
            )));
        }
        let bytes = &self.data[self.position..end];
        self.position = end;
        Ok(bytes)
    }

    pub fn skip(&mut self, count: usize, description: impl Into<String>) -> Result<(), ParseError> {
        self.read_bytes(count, description).map(|_| ())
    }

    pub fn error(&self, operation: impl Into<String>) -> ParseError {
        ParseError::Format {
            file: self.file.clone(),
            offset: self.position,
            section: self.section,
            operation: operation.into(),
            save_version: self.save_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Reader;

    #[test]
    fn primitive_reads_are_little_endian_and_advance_position() {
        let data = [0x12, 0x56, 0x34, 0x04, 0x03, 0x02, 0x01];
        let mut reader = Reader::new(&data, Path::new("test.sav"));

        assert_eq!(reader.read_u8().unwrap(), 0x12);
        assert_eq!(reader.read_u16_le().unwrap(), 0x3456);
        assert_eq!(reader.read_u32_le().unwrap(), 0x0102_0304);
        assert_eq!(reader.position(), data.len());
    }

    #[test]
    fn out_of_bounds_read_reports_context() {
        let data = [0; 2];
        let mut reader = Reader::new(&data, Path::new("short.sav"));
        reader.set_section("MercProfiles");
        reader.set_save_version(102);

        let error = reader.read_u32_le().unwrap_err().to_string();

        assert!(error.contains("short.sav"));
        assert!(error.contains("offset 0x0"));
        assert!(error.contains("MercProfiles"));
        assert!(error.contains("version 102"));
    }
}

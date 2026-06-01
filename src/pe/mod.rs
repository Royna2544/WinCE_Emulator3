use std::{fs, path::Path};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct PeImage {
    pub path: String,
    pub len: usize,
    pub dos_lfanew: u32,
}

impl PeImage {
    pub fn inspect(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let bytes = fs::read(path_ref).map_err(|source| Error::Read {
            path: path_ref.to_path_buf(),
            source,
        })?;

        if bytes.len() < 0x40 || &bytes[..2] != b"MZ" {
            return Err(Error::InvalidArgument(format!(
                "{} is not an MZ PE image",
                path_ref.display()
            )));
        }

        let dos_lfanew =
            u32::from_le_bytes(bytes[0x3c..0x40].try_into().expect("slice length checked"));

        Ok(Self {
            path: path_ref.display().to_string(),
            len: bytes.len(),
            dos_lfanew,
        })
    }
}

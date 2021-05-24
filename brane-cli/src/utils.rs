use anyhow::Result;
use crc32fast::Hasher;
use std::io::Read;
use std::path::PathBuf;
use std::{fs::File, path::Path};

///
///
///
pub fn get_config_dir() -> PathBuf {
    appdirs::user_config_dir(Some("brane"), None, false).expect("Couldn't determine Brane condig directory.")
}

///
///
///
pub fn calculate_crc32(path: &Path) -> Result<u32> {
    let mut file = File::open(&path)?;
    let mut hasher = Hasher::new();

    let chunk_size = 0x004E_2000;
    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = file.by_ref().take(chunk_size as u64).read_to_end(&mut chunk)?;
        if n == 0 {
            break;
        }

        hasher.update(&chunk);
        if n < chunk_size {
            break;
        }
    }

    Ok(hasher.finalize())
}

///
///
///
pub fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

///
///
///
pub fn assert_valid_bakery_name(s: &str) -> Result<()> {
    if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(anyhow!(
            "Invalid name. Must consist only of alphanumeric and/or _ characters."
        ))
    }
}

use anyhow::Result;
use crc32fast::Hasher;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

///
///
///
pub fn calculate_crc32(path: &PathBuf) -> Result<u32> {
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

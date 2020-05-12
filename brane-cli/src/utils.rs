use crc32fast::Hasher;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

type FResult<T> = Result<T, failure::Error>;

///
///
///
pub fn calculate_crc32(path: &PathBuf) -> FResult<u32> {
    let mut file = File::open(&path)?;
    let mut hasher = Hasher::new();

    let chunk_size = 0x4E2000;
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

use std::path::PathBuf;

type FResult<T> = Result<T, failure::Error>;

///
///
///
pub fn handle(
    _context: PathBuf,
    _file: PathBuf,
) -> FResult<()> {
    println!("Build ECU package");

    Ok(())
}

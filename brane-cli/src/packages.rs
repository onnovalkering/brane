type FResult<T> = Result<T, failure::Error>;

///
///
///
pub fn list() -> FResult<()> {
    println!("List packages.");

    Ok(())
}

///
///
///
pub fn remove(_name: String) -> FResult<()> {
    println!("Remove package.");

    Ok(())
}

///
///
///
pub fn test(_name: String) -> FResult<()> {
    println!("Test package.");

    Ok(())
}

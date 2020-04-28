type FResult<T> = Result<T, failure::Error>;

///
///
///
pub fn login(
    _host: String,
    _username: String,
) -> FResult<()> {
    println!("Login.");

    Ok(())
}

///
///
///
pub fn logout(_host: String) -> FResult<()> {
    println!("Logout.");

    Ok(())
}

///
///
///
pub fn pull(_name: String) -> FResult<()> {
    println!("Pull.");

    Ok(())
}

///
///
///
pub fn push(_name: String) -> FResult<()> {
    println!("Push.");

    Ok(())
}

///
///
///
pub fn search(_terms: Vec<String>) -> FResult<()> {
    println!("Search.");

    Ok(())
}

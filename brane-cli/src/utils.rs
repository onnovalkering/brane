use anyhow::Result;
use std::path::PathBuf;

///
///
///
pub fn get_config_dir() -> PathBuf {
    appdirs::user_config_dir(Some("brane"), None, false).expect("Couldn't determine Brane condig directory.")
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

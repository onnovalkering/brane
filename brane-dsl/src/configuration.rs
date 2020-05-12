use appdirs::{user_config_dir, user_data_dir};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Configuration {
    pub registry_api_url: String,
}

///
///
///
pub fn get_config_dir() -> PathBuf {
    user_config_dir(Some("bakery"), None, false).expect("Couldn't determine Bakery config directory.")
}

///
///
///
pub fn get_data_dir() -> PathBuf {
    user_data_dir(Some("bakery"), None, false).expect("Couldn't determine Bakery config directory.")
}

///
///
///
pub fn load() -> Option<Configuration> {
    let config_file = get_config_dir().join("config");
    if !config_file.exists() {
        return None;
    }

    let configuration = fs::read_to_string(config_file).expect("There was a problem reading the configuration file.");

    let configuration: Configuration =
        serde_yaml::from_str(&configuration).expect("There was a problem parsing the configuration file.");

    Some(configuration)
}

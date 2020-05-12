use crate::configuration;
use crate::configuration::Configuration;
use curl::easy::Easy;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input as Prompt, Validator};
use serde_yaml;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use url::Url;

type FResult<T> = Result<T, failure::Error>;

struct RegistryValidator {}

impl Validator for RegistryValidator {
    type Err = failure::Error;

    fn validate(
        &self,
        input: &str,
    ) -> FResult<()> {
        let mut handle = Easy::new();

        ensure!(!input.ends_with('/'), "don't including trailing /");

        let url = Url::parse(&input)?;
        handle.url(url.as_str())?;
        handle.perform()?;

        Ok(())
    }
}

pub fn handle() -> FResult<()> {
    println!("Visit 'https://brane-ri.github.io/bakery' for more details. Press CTRL+C to abort.\n");

    // Create config and data directories
    let config_dir = configuration::get_config_dir();
    let data_dir = configuration::get_data_dir();

    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&data_dir)?;

    // Setup
    let config = gather_configuration()?;

    // Persist configuration (marks successfull setup)
    let config_file = &config_dir.join("config");
    let config_contents = serde_yaml::to_string(&config)?;

    let mut buffer = File::create(config_file).expect("Failed to create config file.");
    write!(buffer, "{}", config_contents).unwrap();

    println!("\nFinished setup.");

    Ok(())
}

///
///
///
fn gather_configuration() -> FResult<Configuration> {
    let registry_url_validator = RegistryValidator {};

    let registry_url: String = Prompt::with_theme(&ColorfulTheme::default())
        .with_prompt("Registry URL")
        .validate_with(registry_url_validator)
        .interact()?;

    // Construct configuration
    let config = Configuration {
        registry_api_url: registry_url,
    };

    Ok(config)
}

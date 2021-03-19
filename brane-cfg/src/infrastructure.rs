use anyhow::Result;
use crate::Secrets;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct InfrastructureDocument {
    locations: HashMap<String, Location>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Location {
    pub kind: String,
    pub address: String,
    pub runtime: String,
    pub credentials: LocationCredentials,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LocationCredentials {
    pub mechanism: String,
    pub username: String,
    pub identity_file: String,
}

impl LocationCredentials {
    pub fn new<S: Into<String>>(mechanism: S, username: S, identity_file: S) -> Self {
        LocationCredentials {
            mechanism: mechanism.into(),
            username: username.into(),
            identity_file: identity_file.into(),
        }
    }

    ///
    ///
    ///
    pub fn resolve_secrets(&self, secrets: &Secrets) -> Result<Self> {
        let username = if self.username.starts_with("s$") {
            secrets.get(&self.username[2..])?
        } else {
            self.username.clone()
        };

        let identity_file = if self.identity_file.starts_with("s$") {
            secrets.get(&self.identity_file[2..])?
        } else {
            self.identity_file.clone()
        };

        Ok(LocationCredentials::new(self.mechanism.clone(), username, identity_file))
    }
}


#[derive(Clone, Debug)]
pub struct Infrastructure {
    store: Store,
}

impl Infrastructure {
    ///
    ///
    ///
    pub fn new<S: Into<String>>(store: S) -> Result<Self> {
        let store = Store::from(store)?;
        Ok(Infrastructure { store })
    }

    ///
    ///
    ///
    pub fn get_location_metadata<S: Into<String>>(&self, location: S) -> Result<Location> {
        let location = location.into();

        if let Store::File(store_file) = &self.store {
            let infra_reader = BufReader::new(File::open(store_file)?);
            let infra_document: InfrastructureDocument = serde_yaml::from_reader(infra_reader)?;

            let metadata = infra_document
                .locations
                .get(&location)
                .map(Location::clone);

            ensure!(metadata.is_some(), "Location '{}' not found in infrastructure metadata.", location);
            let metadata = metadata.unwrap();

            // TODO: validate metadata

            Ok(metadata)
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone, Debug)]
enum Store {
    File(PathBuf),
    Database(Url),
}

impl Store {
    ///
    ///
    ///
    fn from<S: Into<String>>(store: S) -> Result<Self> {
        let store = store.into();

        if let Ok(url) = Url::parse(&store) {
            Ok(Store::Database(url))
        } else {
            let file_path = fs::canonicalize(&store)?;
            Ok(Store::File(file_path))
        }
    }
}

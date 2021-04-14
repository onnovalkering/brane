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
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Location {
    Kube {
        address: String,
        callback_to: String,
        namespace: String,
        credentials: LocationCredentials,
    },
    Local {
        callback_to: String,
        network: String,
    },
    Vm {
        address: String,
        callback_to: String,
        runtime: String,
        credentials: LocationCredentials,
    },
    Slurm {
        address: String,
        callback_to: String,
        runtime: String,
        credentials: LocationCredentials,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "mechanism", rename_all = "kebab-case")]
pub enum LocationCredentials {
    Config {
        file: String,
    },
    SshCertificate {
        username: String,
        certificate: String,
        passphrase: Option<String>
    },
    SshPassword {
        username: String,
        password: String
    },
}

impl LocationCredentials {
    ///
    ///
    ///
    pub fn resolve_secrets(&self, secrets: &Secrets) -> Self {
        use LocationCredentials::*;

        let resolve = |value: &String| {
            if value.starts_with("s$") {
                // Try to resolve secret, but use the value as-is otherwise.
                secrets.get(&value[2..]).unwrap_or_else(|_| value.clone())
            } else {
                value.clone()
            }
        };

        match self {
            Config { file } => {
                let file = resolve(file);

                Config { file }
            }
            SshCertificate { username, certificate, passphrase } => {
                let username = resolve(username);
                let certificate = resolve(certificate);
                let passphrase = passphrase.clone().map(|p| resolve(&p));

                SshCertificate { username, certificate, passphrase}
            }
            SshPassword { username, password } => {
                let username = resolve(username);
                let password = resolve(password);

                SshPassword { username, password }
            }
        }
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

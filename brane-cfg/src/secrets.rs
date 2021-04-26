use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::BufReader;
use std::fs::{self, File};
use std::path::PathBuf;
use url::Url;

#[derive(Clone, Debug)]
pub struct Secrets {
    store: Store,
}

impl Secrets {
    ///
    ///
    ///
    pub fn new<S: Into<String>>(store: S) -> Result<Self> {
        let store = Store::from(store)?;
        Ok(Secrets { store })
    }

    ///
    ///
    ///
    pub fn validate(&self) -> Result<()> {
        if let Store::File(store_file) = &self.store {
            let infra_reader = BufReader::new(File::open(store_file)?);
            let _: HashMap<String, String> = serde_yaml::from_reader(infra_reader)
                .context("Secrets file is not valid.")?;

            Ok(())
        } else {
            unreachable!()
        }
    }

    ///
    ///
    ///
    pub fn get<S: Into<String>>(&self, secret_key: S) -> Result<String> {
        let secret_key = secret_key.into();

        if let Store::File(store_file) = &self.store {
            let secrets_reader = BufReader::new(File::open(store_file)?);
            let secrets_document: HashMap<String, String> = serde_yaml::from_reader(secrets_reader)
                .with_context(|| format!("Error while deserializing file: {:?}", store_file))?;

            let secret = secrets_document
                .get(&secret_key)
                .map(String::clone);

            secret.ok_or(anyhow!("Secret '{}' not in secrets store.", secret_key))
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

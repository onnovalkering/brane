use anyhow::{Context, Result};
use hashicorp_vault::{client::TokenData, Client};
use specifications::common::Value;

type Map<T> = std::collections::HashMap<String, T>;

pub trait Vault {
    fn exists(
        &self,
        name: &str,
    ) -> bool;

    fn get(
        &self,
        name: &str,
    ) -> Result<Value>;
}

///
///
///
pub struct HashiVault {
    client: Client<TokenData>,
}

impl HashiVault {
    ///
    ///
    ///
    pub fn new(
        host: String,
        token: String,
    ) -> Self {
        let client = Client::new(host, token).unwrap();

        HashiVault { client }
    }
}

impl Vault for HashiVault {
    ///
    ///
    ///
    fn exists(
        &self,
        name: &str,
    ) -> bool {
        self.get(name).is_ok()
    }

    ///
    ///
    ///
    fn get(
        &self,
        name: &str,
    ) -> Result<Value> {
        if let Ok(secret) = self.client.get_secret(name) {
            Ok(Value::Unicode(secret))
        } else {
            bail!("Failed to get secret: {}", name)
        }
    }
}

///
///
///
#[derive(Clone)]
pub struct InMemoryVault {
    secrets: Map<Value>,
}

impl InMemoryVault {
    ///
    ///
    ///
    pub fn new(secrets: Map<Value>) -> Self {
        InMemoryVault { secrets }
    }
}

impl Vault for InMemoryVault {
    ///
    ///
    ///
    fn exists(
        &self,
        name: &str,
    ) -> bool {
        self.secrets.contains_key(name)
    }

    ///
    ///
    ///
    fn get(
        &self,
        name: &str,
    ) -> Result<Value> {
        Ok(self
            .secrets
            .get(name)
            .with_context(|| format!("Trying to access undefined secret: {}", name))?
            .clone())
    }
}

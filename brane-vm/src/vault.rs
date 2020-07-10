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
    ) -> Value;
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
    ) -> Value {
        self.secrets
            .get(name)
            .expect("Trying to access undefined secret.")
            .clone()
    }
}

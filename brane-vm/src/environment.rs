use specifications::common::Value;
use std::rc::Rc;

type Map<T> = std::collections::HashMap<String, T>;

pub trait Environment {
    fn exists(
        &self,
        name: &str,
    ) -> bool;

    fn get(
        &self,
        name: &str,
    ) -> Value;

    fn set(
        &mut self,
        name: &str,
        value: &Value,
    );

    fn remove(
        &mut self,
        name: &str,
    );

    fn child(&self) -> Box<dyn Environment>;

    fn variables(&self) -> Map<Value>;
}

///
///
///
#[derive(Clone)]
pub struct InMemoryEnvironment {
    variables: Map<Value>,
    parent: Option<Rc<dyn Environment>>,
}

impl InMemoryEnvironment {
    ///
    ///
    ///
    pub fn new(
        arguments: Option<Map<Value>>,
        parent: Option<Rc<dyn Environment>>,
    ) -> Self {
        let variables = arguments.unwrap_or_else(Map::<Value>::new);

        InMemoryEnvironment { variables, parent }
    }
}

impl Environment for InMemoryEnvironment {
    ///
    ///
    ///
    fn exists(
        &self,
        name: &str,
    ) -> bool {
        if self.variables.contains_key(name) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.exists(name);
        }

        false
    }

    ///
    ///
    ///
    fn get(
        &self,
        name: &str,
    ) -> Value {
        let variable = self.variables.get(name);
        if let Some(variable) = variable {
            return variable.clone();
        }

        if let Some(parent) = &self.parent {
            return parent.get(name);
        }

        panic!("Trying to access undeclared variable.");
    }

    ///
    ///
    ///
    fn set(
        &mut self,
        name: &str,
        value: &Value,
    ) {
        self.variables.insert(name.to_string(), value.clone());
    }

    ///
    ///
    ///
    fn remove(
        &mut self,
        name: &str,
    ) {
        self.variables.remove(name);
    }

    ///
    ///
    ///
    fn child(&self) -> Box<dyn Environment> {
        let current = Rc::new(self.clone());
        let environment = InMemoryEnvironment::new(None, Some(current));

        Box::new(environment)
    }

    ///
    ///
    ///
    fn variables(&self) -> Map<Value> {
        self.variables.clone()
    }
}

use specifications::common::Value;
use std::rc::Rc;
use std::cell::RefCell;
use redis::{Connection, Client};
use redis::Commands;
use serde_json;

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

        panic!("Trying to access undeclared variable: {}", name);
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

//
///
///
#[derive(Clone)]
pub struct RedisEnvironment {
    connection: Rc<RefCell<Connection>>,
    parent: Option<Rc<dyn Environment>>,
    prefix: String,
}

impl RedisEnvironment {
    ///
    ///
    ///
    pub fn new(
        prefix: String,
        parent: Option<Rc<dyn Environment>>,
        client: &Client,
    ) -> Self {
        let connection = client.get_connection().unwrap();
        RedisEnvironment { connection: Rc::new(RefCell::new(connection)), parent, prefix }
    }

    pub fn local_exists(
        &self,
        key: &str,
    ) -> bool {
        self.connection.borrow_mut().exists(format!("{}_{}", self.prefix, key)).unwrap()
    }

    ///
    ///
    ///
    pub fn local_get(
        &self,
        key: &str
    ) -> Option<String> {
        if let Ok(result) = self.connection.borrow_mut().get(format!("{}_{}", self.prefix, key)) {
            Some(result)
        } else {
            None
        }
    }

    pub fn local_remove(
        &self,
        key: &str
    ) -> () {
        self.connection.borrow_mut().del(format!("{}_{}", self.prefix, key)).unwrap()
    }

    ///
    ///
    ///
    pub fn local_set(
        &self,
        key: &str,
        value: String
    ) -> String {
        self.connection.borrow_mut().set(format!("{}_{}", self.prefix, key), value).unwrap()
    }
}

impl Environment for RedisEnvironment {
    ///
    ///
    ///
    fn exists(
        &self,
        name: &str,
    ) -> bool {
        if self.local_exists(name) {
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
        let value_json = self.local_get(name);
        if let Some(value_json) = value_json {
            return serde_json::from_str(&value_json).unwrap();
        }

        if let Some(parent) = &self.parent {
            return parent.get(name)
        }

        panic!("Trying to access undeclared variable: '{}'.", name);
    }

    ///
    ///
    ///
    fn set(
        &mut self,
        name: &str,
        value: &Value,
    ) {
        let value_json = serde_json::to_string(value).unwrap();
        self.local_set(name, value_json);
    }

    ///
    ///
    ///
    fn remove(
        &mut self,
        name: &str,
    ) {
        self.local_remove(name);
    }

    ///
    ///
    ///
    fn child(&self) -> Box<dyn Environment> {
        unimplemented!()
    }

    ///
    ///
    ///
    fn variables(&self) -> Map<Value> {
        let mut variables = Map::<Value>::new();

        let keys: Vec<String>;
        {
            let mut conn = self.connection.borrow_mut();
            keys = conn.scan_match(format!("{}*", self.prefix)).unwrap().collect();
        }

        for key in keys {
            let name = String::from(&key[self.prefix.len()+1..]);
            let value = self.get(&name);

            variables.insert(name, value);
        }

        variables
    }
}
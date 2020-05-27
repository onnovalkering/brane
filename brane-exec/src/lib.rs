#[macro_use]
extern crate log;

pub mod docker;

type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub struct ExecuteInfo {
    pub image: String,
    pub payload: String,
}

impl ExecuteInfo {
    ///
    ///
    ///
    pub fn new(image: String, payload: String) -> Self {
        ExecuteInfo { image, payload }
    }
}


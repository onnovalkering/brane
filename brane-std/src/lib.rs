#[macro_use]
extern crate lazy_static;

mod fs;
mod text;

use anyhow::Result;
use brane_sys::System;
use specifications::common::Value;
use specifications::package::PackageInfo;

type Map<T> = std::collections::HashMap<String, T>;
type Func = fn(&Map<Value>, &Box<dyn System>) -> Result<Value>;

lazy_static! {
    pub static ref PACKAGES: Map<PackageInfo> = {
        let mut packages = Map::new();
        packages.insert(fs::PACKAGE.name.clone(), fs::PACKAGE.clone());
        packages.insert(text::PACKAGE.name.clone(), text::PACKAGE.clone());

        packages
    };
    pub static ref FUNCTIONS: Map<Map<Func>> = {
        let mut functions = Map::new();
        functions.insert(fs::PACKAGE.name.clone(), fs::FUNCTIONS.clone());
        functions.insert(text::PACKAGE.name.clone(), text::FUNCTIONS.clone());

        functions
    };
}

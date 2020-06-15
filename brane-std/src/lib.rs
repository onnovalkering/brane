#[macro_use]
extern crate lazy_static;

mod text;

use anyhow::Result;
use specifications::common::Value;
use specifications::package::PackageInfo;

type Map<T> = std::collections::HashMap<String, T>;
type Function = fn(&Map<Value>) -> Result<Value>;

lazy_static! {
    pub static ref PACKAGES: Map<PackageInfo> = {
        let mut packages = Map::new();
        packages.insert(text::PACKAGE.name.clone(), text::PACKAGE.clone());

        packages
    };
    pub static ref FUNCTIONS: Map<Map<Function>> = {
        let mut functions = Map::new();
        functions.insert(text::PACKAGE.name.clone(), text::FUNCTIONS.clone());

        functions
    };
}

use anyhow::Result;
use brane_sys::System;
use rand::distributions::Alphanumeric;
use rand::Rng;
use specifications::common::{CallPattern, Function, Parameter, Property, Type, Value};
use specifications::package::PackageInfo;
use url::Url;

type Map<T> = std::collections::HashMap<String, T>;
type Func = fn(&Map<Value>, &Box<dyn System>) -> Result<Value>;

lazy_static! {
    pub static ref PACKAGE: PackageInfo = {
        let name = String::from("filesystem");
        let version = env!("CARGO_PKG_VERSION").into();
        let kind = String::from("std");

        let mut functions = Map::<Function>::new();
        let mut types = Map::<Type>::new();

        // Directory
        let directory = Type::new(String::from("Directory"), vec![
            Property::new_quick("url", "string")
        ]);

        // File
        let file = Type::new(String::from("File"), vec![
            Property::new_quick("url", "string")
        ]);

        // new_directory
        let new_directory = Function::new(vec![], None, directory.name.clone());
        functions.insert(String::from("new_directory"), new_directory);

        // new_directory <name>
        let new_directory_name = Function::new(
            vec![
                Parameter::new(String::from("name"), String::from("string"), None, None, None),
            ],
            Some(CallPattern::new(Some(String::from("new_directory")), None, None)),
            directory.name.clone()
        );
        functions.insert(String::from("new_directory_name"), new_directory_name);

        // new_directory <name> in <parent>
        let new_directory_in = Function::new(
            vec![
                Parameter::new(String::from("name"), String::from("string"), None, None, None),
                Parameter::new(String::from("parent"), String::from("Directory"), None, None, None),
            ],
            Some(CallPattern::new(Some(String::from("new_directory")), Some(vec![String::from("in")]), None)),
            directory.name.clone()
        );
        functions.insert(String::from("new_directory_in"), new_directory_in);

        // new_file
        let new_file = Function::new(vec![], None, String::from("File"));
        functions.insert(String::from("new_file"), new_file);

        // new_file <name>
        let new_file_name = Function::new(
            vec![
                Parameter::new(String::from("name"), String::from("string"), None, None, None),
            ],
            Some(CallPattern::new(Some(String::from("new_file")), None, None)),
            file.name.clone());
        functions.insert(String::from("new_file_name"), new_file_name);

        // new_file <name> in <parent>
        let new_file_in = Function::new(
            vec![
                Parameter::new(String::from("name"), String::from("string"), None, None, None),
                Parameter::new(String::from("parent"), String::from("Directory"), None, None, None),
            ],
            Some(CallPattern::new(Some(String::from("new_file")), Some(vec![String::from("in")]), None)),
            file.name.clone()
        );
        functions.insert(String::from("new_file_in"), new_file_in);

        // new_temp_directory
        let new_directory = Function::new(vec![], None, directory.name.clone());
        functions.insert(String::from("new_temp_directory"), new_directory);

        // new_temp_file
        let new_file = Function::new(vec![], None, String::from("File"));
        functions.insert(String::from("new_temp_file"), new_file);

        types.insert(directory.name.clone(), directory);
        types.insert(file.name.clone(), file);

        PackageInfo::new(name, version, None, kind, Some(functions), Some(types))
    };

    // Matching is performed in the order as stated below
    pub static ref FUNCTIONS: Map<Func> = {
        let mut functions = Map::new();
        functions.insert(String::from("new_directory_in"), new_directory as Func);
        functions.insert(String::from("new_directory_name"), new_directory as Func);
        functions.insert(String::from("new_directory"), new_directory as Func);
        functions.insert(String::from("new_file_in"), new_file as Func);
        functions.insert(String::from("new_file_name"), new_file as Func);
        functions.insert(String::from("new_file"), new_file as Func);
        functions.insert(String::from("new_temp_directory"), new_temp_directory as Func);
        functions.insert(String::from("new_temp_file"), new_temp_file as Func);

        functions
    };
}

///
///
///
pub fn new_directory(
    arguments: &Map<Value>,
    system: &Box<dyn System>,
) -> Result<Value> {
    let name = arguments
        .get("name")
        .map(|v| v.as_string().unwrap())
        .unwrap_or_else(|| gen_random_name());

    let parent = arguments
        .get("parent")
        .map(|v| {
            if let Value::Struct { properties, .. } = v {
                let url = properties.get("url").unwrap().as_string().unwrap();
                let url = Url::parse(&url).unwrap();
                Some(url)
            } else {
                None
            }
        })
        .unwrap_or_else(|| None);

    let directory_url = system.create_dir(&name, parent.as_ref(), false)?;
    let directory_url = Value::Unicode(directory_url.as_str().to_string());

    let mut properties = Map::<Value>::new();
    properties.insert(String::from("url"), directory_url);

    let directory = Value::Struct {
        data_type: String::from("Directory"),
        properties,
    };

    Ok(directory)
}

///
///
///
pub fn new_file(
    arguments: &Map<Value>,
    system: &Box<dyn System>,
) -> Result<Value> {
    let name = arguments
        .get("name")
        .map(|v| v.as_string().unwrap())
        .unwrap_or_else(|| gen_random_name());

    let parent = arguments
        .get("parent")
        .map(|v| {
            if let Value::Struct { properties, .. } = v {
                let url = properties.get("url").unwrap().as_string().unwrap();
                let url = Url::parse(&url).unwrap();
                Some(url)
            } else {
                None
            }
        })
        .unwrap_or_else(|| None);

    let file_url = system.create_file(&name, parent.as_ref(), false)?;
    let file_url = Value::Unicode(file_url.as_str().to_string());

    let mut properties = Map::<Value>::new();
    properties.insert(String::from("url"), file_url);

    let file = Value::Struct {
        data_type: String::from("File"),
        properties,
    };

    Ok(file)
}

///
///
///
pub fn new_temp_directory(
    arguments: &Map<Value>,
    system: &Box<dyn System>,
) -> Result<Value> {
    let name = arguments
        .get("name")
        .map(|v| v.as_string().unwrap())
        .unwrap_or_else(|| gen_random_name());

    let parent = None;
    let directory_url = system.create_dir(&name, parent, true)?;
    let directory_url = Value::Unicode(directory_url.as_str().to_string());

    let mut properties = Map::<Value>::new();
    properties.insert(String::from("url"), directory_url);

    let directory = Value::Struct {
        data_type: String::from("Directory"),
        properties,
    };

    Ok(directory)
}

///
///
///
pub fn new_temp_file(
    arguments: &Map<Value>,
    system: &Box<dyn System>,
) -> Result<Value> {
    let name = arguments
        .get("name")
        .map(|v| v.as_string().unwrap())
        .unwrap_or_else(|| gen_random_name());

    let parent = None;
    let file_url = system.create_file(&name, parent, true)?;
    let file_url = Value::Unicode(file_url.as_str().to_string());

    let mut properties = Map::<Value>::new();
    properties.insert(String::from("url"), file_url);

    let file = Value::Struct {
        data_type: String::from("File"),
        properties,
    };

    Ok(file)
}

///
///
///
fn gen_random_name() -> String {
    let random_name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .collect::<String>()
        .to_lowercase();

    random_name
}

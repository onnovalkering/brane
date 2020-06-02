use brane_vm::{environment::InMemoryEnvironment, machine::Machine};
use chrono::Utc;
use console::{pad_str, Alignment};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use dialoguer::Input as Prompt;
use indicatif::HumanDuration;
use prettytable::format::FormatBuilder;
use prettytable::Table;
use semver::Version;
use serde_yaml;
use specifications::common::Value;
use specifications::instructions::Instruction;
use specifications::package::PackageInfo;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;

///
///
///
pub fn get_packages_dir() -> PathBuf {
    appdirs::user_data_dir(Some("brane"), None, false)
        .expect("Couldn't determine Brane data directory.")
        .join("packages")
}

///
///
///
pub fn get_package_dir(
    name: &str,
    version: Option<&str>,
) -> FResult<PathBuf> {
    let packages_dir = get_packages_dir();
    let package_dir = packages_dir.join(&name);

    if version.is_none() {
        return Ok(package_dir);
    }

    let version = version.unwrap();
    let version = if version == "latest" {
        ensure!(package_dir.exists(), "Package does not exist.");

        let versions = fs::read_dir(&package_dir)?;
        let mut versions: Vec<Version> = versions
            .map(|v| v.unwrap().file_name())
            .map(|v| Version::parse(&v.into_string().unwrap()).unwrap())
            .collect();

        versions.sort();
        versions.reverse();

        versions[0].to_string()
    } else {
        Version::parse(&version)
            .expect("Not a valid semantic version.")
            .to_string()
    };

    Ok(package_dir.join(version))
}

///
///
///
pub fn list() -> FResult<()> {
    let packages_dir = get_packages_dir();
    if !packages_dir.exists() {
        println!("No packages found.");
        return Ok(());
    }

    // Prepare display table.
    let format = FormatBuilder::new()
        .column_separator('\0')
        .borders('\0')
        .padding(1, 1)
        .build();

    let mut table = Table::new();
    table.set_format(format);
    table.add_row(row!["ID", "NAME", "VERSION", "KIND", "CREATED"]);

    // Add a row to the table for each version of each group.
    let packages = fs::read_dir(packages_dir)?;
    for package in packages {
        let package_path = package?.path();
        if !package_path.is_dir() {
            continue;
        }

        let versions = fs::read_dir(package_path)?;
        for version in versions {
            let path = version?.path();
            let package_file = path.join("package.yml");

            if !path.is_dir() || !package_file.exists() {
                continue;
            }

            let now = Utc::now().timestamp();
            if let Ok(package_info) = PackageInfo::from_path(package_file) {
                let uuid = format!("{}", &package_info.id);

                let id = pad_str(&uuid[..8], 10, Alignment::Left, Some(".."));
                let name = pad_str(&package_info.name, 20, Alignment::Left, Some(".."));
                let version = pad_str(&package_info.version, 10, Alignment::Left, Some(".."));
                let kind = pad_str(&package_info.kind, 10, Alignment::Left, Some(".."));
                let elapsed = Duration::from_secs((now - package_info.created.timestamp()) as u64);
                let created = format!("{} ago", HumanDuration(elapsed));
                let created = pad_str(&created, 15, Alignment::Left, None);

                table.add_row(row![id, name, version, kind, created]);
            }
        }
    }

    table.printstd();

    Ok(())
}

///
///
///
pub fn remove(
    name: String,
    version: Option<String>,
    force: bool,
) -> FResult<()> {
    // Remove without confirmation if explicity stated package version.
    if let Some(version) = version {
        let package_dir = get_package_dir(&name, Some(&version))?;
        if fs::remove_dir_all(&package_dir).is_err() {
            println!("No package with name '{}' and version '{}' exists!", name, version);
        }

        return Ok(());
    }

    let package_dir = get_package_dir(&name, None)?;
    if !package_dir.exists() {
        println!("No package with name '{}' exists!", name);
        return Ok(());
    }

    // Also remove without confirmation if --force is provided.
    if force {
        fs::remove_dir_all(&package_dir)?;
        return Ok(());
    }

    // Look for packages.
    let versions = fs::read_dir(&package_dir)?
        .map(|v| v.unwrap().file_name())
        .map(|v| String::from(v.to_string_lossy()));

    println!("Do you want to remove the following version(s)?");
    for version in versions {
        println!("- {}", version);
    }
    println!();

    if Confirm::new().interact()? {
        fs::remove_dir_all(&package_dir)?;
    }

    Ok(())
}

///
///
///
pub fn test(
    name: String,
    version: Option<String>,
) -> FResult<()> {
    let version_or_latest = version.unwrap_or_else(|| String::from("latest"));
    let package_dir = get_package_dir(&name, Some(&version_or_latest))?;
    ensure!(package_dir.exists(), "No package found.");

    let package_info = PackageInfo::from_path(package_dir.join("package.yml"))?;
    match package_info.kind.as_str() {
        "dsl" => test_dsl(package_dir, package_info),
        "ecu" => test_ecu(package_dir, package_info),
        _ => bail!("Only testing of DSL and ECU packages is supported."),
    }
}

///
///
///
fn test_dsl(
    package_dir: PathBuf,
    _package_info: PackageInfo,
) -> FResult<()> {
    let instructions_file = package_dir.join("instructions.yml");
    ensure!(instructions_file.exists(), "No instructions found.");

    // Load instructions
    let buf_reader = BufReader::new(File::open(instructions_file)?);
    let mut instructions: Vec<Instruction> = serde_yaml::from_reader(buf_reader)?;

    let mut arguments = Map::<Value>::new();
    test_dsl_preprocess_instructions(&mut instructions, &mut arguments)?;

    let environment = InMemoryEnvironment::new(Some(arguments), None);
    let mut machine = Machine::new(Box::new(environment));
    let output = machine.interpret(instructions)?;

    if let Some(value) = output {
        if let Value::Pointer { variable, .. } = value {
            let value = machine.environment.get(&variable);
            println!("\n{:?}", value);
        } else {
            println!("\n{:?}", value);
        }
    }

    Ok(())
}

///
///
///
fn test_dsl_preprocess_instructions(
    instructions: &mut Vec<Instruction>,
    arguments: &mut Map<Value>,
) -> FResult<()> {
    for instruction in instructions {
        match instruction {
            Instruction::Act(act) => {
                let name = act.meta.get("name").expect("No `name` property in metadata.");
                let version = act.meta.get("version").expect("No `version` property in metadata.");

                let package_dir = get_package_dir(&name, Some(version))?;
                let image_file = package_dir.join("image.tar");
                if image_file.exists() {
                    act.meta
                        .insert(String::from("image_file"), String::from(image_file.to_string_lossy()));
                }
            }
            Instruction::Sub(sub) => {
                test_dsl_preprocess_instructions(&mut sub.instructions, arguments)?;
            }
            Instruction::Var(var) => {
                for get in &var.get {
                    let value = match get.data_type.as_str() {
                        "boolean" => Value::Boolean(prompt(&get.name, &get.data_type)?),
                        "integer" => Value::Integer(prompt(&get.name, &get.data_type)?),
                        "real" => Value::Real(prompt(&get.name, &get.data_type)?),
                        "unicode" => Value::Unicode(prompt(&get.name, &get.data_type)?),
                        _ => unimplemented!(),
                    };

                    arguments.insert(get.name.clone(), value);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

///
///
///
fn test_ecu(
    package_dir: PathBuf,
    package_info: PackageInfo,
) -> FResult<()> {
    let image_tag = format!("{}:{}", package_info.name, package_info.version);
    let image_file = package_dir.join("image.tar");
    ensure!(image_file.exists(), "No image found.");

    // Load image
    let output = Command::new("docker")
        .arg("load")
        .arg("-i")
        .arg(image_file)
        .output()
        .expect("Couldn't run 'docker' command.");

    ensure!(output.status.success(), "Failed to load image.");

    // Run image
    Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-it")
        .arg(&image_tag)
        .arg("test")
        .status()
        .expect("Couldn't run 'docker' command.");

    // Unload image
    let output = Command::new("docker")
        .arg("image")
        .arg("rm")
        .arg(&image_tag)
        .output()
        .expect("Couldn't run 'docker' command.");

    if !output.status.success() {
        warn!("Failed to unload '{}', image remains loaded in Docker.", image_tag);
    }

    Ok(())
}

///
///
///
fn prompt<T>(
    name: &str,
    data_type: &str,
) -> std::io::Result<T>
where
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    Prompt::with_theme(&ColorfulTheme::default())
        .with_prompt(&format!("{} ({})", name, data_type))
        .interact()
}

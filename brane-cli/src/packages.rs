use crate::docker;
use anyhow::Result;
use bollard::errors::Error;
use bollard::image::ImportImageOptions;
use bollard::image::TagImageOptions;
use bollard::models::BuildInfo;
use bollard::Docker;
use chrono::Utc;
use console::{pad_str, Alignment};
use dialoguer::Confirm;
use futures_util::stream::TryStreamExt;
use hyper::Body;
use indicatif::HumanDuration;
use prettytable::format::FormatBuilder;
use prettytable::Table;
use semver::Version;
use serde_json::json;
use specifications::package::PackageIndex;
use specifications::package::PackageInfo;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File as TFile;
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};

///
///
///
pub fn get_package_dir(
    name: &str,
    version: Option<&str>,
) -> Result<PathBuf> {
    let packages_dir = get_packages_dir();
    let package_dir = packages_dir.join(&name);

    if version.is_none() {
        return Ok(package_dir);
    }

    let version = version.unwrap();
    let version = if version == "latest" {
        if !package_dir.exists() {
            return Err(anyhow!("Package not found."));
        }

        let versions = fs::read_dir(&package_dir)?;
        let mut versions: Vec<Version> = versions
            .map(|v| v.unwrap().file_name())
            .map(|v| Version::parse(&v.into_string().unwrap()).unwrap())
            .collect();

        versions.sort();
        versions.reverse();

        versions[0].to_string()
    } else {
        Version::parse(version)
            .expect("Not a valid semantic version.")
            .to_string()
    };

    Ok(package_dir.join(version))
}

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
pub fn get_package_index() -> Result<PackageIndex> {
    let packages_dir = get_packages_dir();
    if !packages_dir.exists() {
        return Ok(PackageIndex::empty());
    }

    let mut packages = vec![];
    for package in fs::read_dir(packages_dir)? {
        let package_path = package?.path();
        if !package_path.is_dir() {
            continue;
        }

        let versions = fs::read_dir(package_path)?;
        for version in versions {
            let path = version?.path();
            let package_file = path.join("package.yml");
            let lock_file = path.join(".lock");

            if !path.is_dir() || !package_file.exists() || lock_file.exists() {
                continue;
            }

            if let Ok(package_info) = PackageInfo::from_path(package_file) {
                packages.push(package_info);
            }
        }
    }

    PackageIndex::from_value(json!(packages))
}

///
///
///
pub fn inspect(
    name: String,
    version: String,
) -> Result<()> {
    let package_dir = get_package_dir(&name, Some(version).as_deref())?;
    let package_file = package_dir.join("package.yml");

    if let Ok(package_info) = PackageInfo::from_path(package_file) {
        println!("{:#?}", package_info);
    } else {
        return Err(anyhow!("Failed to read package information."));
    }

    Ok(())
}

///
///
///
pub fn list() -> Result<()> {
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
            let lock_file = path.join(".lock");

            if !path.is_dir() || !package_file.exists() || lock_file.exists() {
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
pub async fn load(
    name: String,
    version: Option<String>,
) -> Result<()> {
    let version_or_latest = version.unwrap_or_else(|| String::from("latest"));
    let package_dir = get_package_dir(&name, Some(&version_or_latest))?;
    if !package_dir.exists() {
        return Err(anyhow!("Package not found."));
    }

    let package_info = PackageInfo::from_path(package_dir.join("package.yml"))?;
    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = package_dir.join("image.tar");

    let docker = Docker::connect_with_local_defaults()?;

    // Abort, if image is already loaded
    if docker.inspect_image(&image).await.is_ok() {
        println!("Image already exists in local Docker deamon.");
        return Ok(());
    }

    println!("Image doesn't exist in Docker deamon: importing...");
    let options = ImportImageOptions { quiet: true };

    let file = TFile::open(image_file).await?;
    let byte_stream = FramedRead::new(file, BytesCodec::new()).map(|r| {
        let bytes = r.unwrap().freeze();
        Ok::<_, Error>(bytes)
    });

    let body = Body::wrap_stream(byte_stream);
    let result = docker.import_image(options, body, None).try_collect::<Vec<_>>().await?;
    if let Some(BuildInfo {
        stream: Some(stream), ..
    }) = result.first()
    {
        let (_, image_hash) = stream
            .trim()
            .split_once("sha256:")
            .expect("Expected image hash in load output.");
        debug!("Imported image: {}", image_hash);

        let options = TagImageOptions {
            repo: &package_info.name,
            tag: &package_info.version,
        };

        docker.tag_image(image_hash, Some(options)).await?;
    }

    Ok(())
}

///
///
///
pub async fn remove(
    name: String,
    version: Option<String>,
    force: bool,
) -> Result<()> {
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

    // Look for packages.
    let versions = fs::read_dir(&package_dir)?
        .map(|v| v.unwrap().file_name())
        .map(|v| String::from(v.to_string_lossy()))
        .collect::<Vec<String>>();

    // Ask for permission, if --force is not provided
    if !force {
        println!("Do you want to remove the following version(s)?");
        for version in &versions {
            println!("- {}", version);
        }
        println!();

        // Abort, if not approved
        if !Confirm::new().interact()? {
            return Ok(());
        }
    }

    // Check if image is locally loaded in Docker
    for version in &versions {
        let image_name = format!("{}:{}", name, version);
        docker::remove_image(&image_name).await?;

        let image_name = format!("localhost:5000/library/{}:{}", name, version);
        docker::remove_image(&image_name).await?;
    }

    fs::remove_dir_all(&package_dir)?;

    Ok(())
}

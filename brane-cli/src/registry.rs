use crate::packages;
use crate::utils;
use anyhow::{Context, Result};
use console::style;
use console::{pad_str, Alignment};
use dialoguer::Confirm;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::format::FormatBuilder;
use prettytable::Table;
use reqwest::{self, multipart::Form, multipart::Part, Body, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JValue};
use serde_with::skip_serializing_none;
use specifications::package::PackageIndex;
use specifications::package::PackageInfo;
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use tar::Archive;
use tokio::fs::File as TokioFile;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
struct RegistryConfig {
    pub url: String,
    pub username: String,
}

impl RegistryConfig {
    fn new() -> Self {
        RegistryConfig {
            url: Default::default(),
            username: Default::default(),
        }
    }

    fn from_path(path: &PathBuf) -> Result<RegistryConfig> {
        let config_reader = BufReader::new(File::open(path)?);
        let config = serde_yaml::from_reader(config_reader)?;

        Ok(config)
    }
}

///
///
///
pub fn login(
    url: String,
    username: String,
) -> Result<()> {
    let url = Url::parse(&url).with_context(|| format!("Not a valid absolute URL: {}", url))?;

    let host = url
        .host_str()
        .with_context(|| format!("URL does not have a (valid) host: {}", url))?;

    let config_file = utils::get_config_dir().join("registry.yml");
    let mut config = if config_file.exists() {
        RegistryConfig::from_path(&config_file)?
    } else {
        RegistryConfig::new()
    };

    config.username = username;
    config.url = format!("{}://{}:{}", url.scheme(), host, url.port().unwrap_or(8080));

    // Write registry.yml to config directory
    fs::create_dir_all(&config_file.parent().unwrap())?;
    let mut buffer = File::create(config_file)?;
    write!(buffer, "{}", serde_yaml::to_string(&config)?)?;

    Ok(())
}

///
///
///
pub fn logout() -> Result<()> {
    let config_file = utils::get_config_dir().join("registry.yml");
    if config_file.exists() {
        fs::remove_file(config_file)?;
    }

    Ok(())
}

///
///
///
pub async fn pull(
    name: String,
    version: Option<String>,
) -> Result<()> {
    let version = version.expect("please provide version");

    let url = get_registry_endpoint(format!("/{}/{}/archive", name, version))?;
    let mut package_archive = reqwest::get(&url).await?;

    let content_length = package_archive
        .headers()
        .get("content-length")
        .unwrap()
        .to_str()?
        .parse()?;

    // Write package archive to temporary file
    let temp_filepath = env::temp_dir().join("archive.tar.gz");
    let mut temp_file = File::create(&temp_filepath)?;

    let progress = ProgressBar::new(content_length);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("Downloading... [{elapsed_precise}] {bar:40.cyan/blue} {percent}/100%")
            .progress_chars("##-"),
    );

    while let Some(chunk) = package_archive.chunk().await? {
        progress.inc(chunk.len() as u64);
        temp_file.write_all(&chunk)?; // If causes bug, use .write(&chunk)
    }

    progress.finish();

    // Unpack temporary file to target location
    let archive_file = File::open(&temp_filepath)?;
    let package_dir = packages::get_package_dir(&name, Some(&version))?;
    fs::create_dir_all(&package_dir)?;

    let progress = ProgressBar::new(content_length);
    progress.set_style(ProgressStyle::default_bar().template("Extracting...  [{elapsed_precise}]"));
    progress.enable_steady_tick(250);

    let mut archive = Archive::new(GzDecoder::new(archive_file));
    archive.unpack(&package_dir)?;

    progress.finish();

    // Remove temporary file
    if let Err(_) = fs::remove_file(&temp_filepath) {
        warn!("Failed to remove temporary file: {:?}", temp_filepath);
    }

    println!(
        "\nSuccessfully pulled version {} of package {}.",
        style(&version).bold().cyan(),
        style(&name).bold().cyan(),
    );

    Ok(())
}

///
///
///
pub async fn push(
    name: String,
    version: String,
) -> Result<()> {
    let package_dir = packages::get_package_dir(&name, Some(&version))?;
    let archive_filename = "archive.tar.gz";
    let archive_filepath = env::temp_dir().join(archive_filename);
    let archive_file = File::create(&archive_filepath)?;

    let progress = ProgressBar::new(0);
    progress.set_style(ProgressStyle::default_bar().template("Compressing... [{elapsed_precise}]"));
    progress.enable_steady_tick(250);

    // Create package tarball
    let gz = GzEncoder::new(&archive_file, Compression::fast());
    let mut tar = tar::Builder::new(gz);
    tar.append_dir_all(".", package_dir)?;
    tar.into_inner()?;

    // Calcualte checksum
    let checksum = utils::calculate_crc32(&archive_filepath)?;

    progress.finish();

    // Upload file
    let url = get_registry_endpoint(format!("/{}/{}?checksum={}", name, version, checksum))?;
    let request = Client::new().post(&url);

    let file = TokioFile::open(&archive_filepath).await?;
    let file = FramedRead::new(file, BytesCodec::new());
    let reader = Body::wrap_stream(file);

    let mut form = Form::new();
    form = form.part("file", Part::stream(reader).file_name(archive_filename));

    let progress = ProgressBar::new(0);
    progress.set_style(ProgressStyle::default_bar().template("Uploading...   [{elapsed_precise}]"));
    progress.enable_steady_tick(250);

    let request = request.multipart(form);
    let response = request.send().await?;

    let response_status = response.status();

    progress.finish();

    if response_status.is_success() {
        println!(
            "\nSuccessfully pushed version {} of package {}.",
            style(&version).bold().cyan(),
            style(&name).bold().cyan(),
        );
    } else {
        let response_text = response.text().await?;
        println!("\nFailed to push package: {}", response_text)
    }

    Ok(())
}

///
///
///
pub async fn search(term: String) -> Result<()> {
    let url = get_registry_endpoint(format!("?t={}", term))?;
    let packages: Vec<PackageInfo> = reqwest::get(&url).await?.json().await?;

    // Prepare display table
    let format = FormatBuilder::new()
        .column_separator('\0')
        .borders('\0')
        .padding(1, 1)
        .build();

    let mut table = Table::new();
    table.set_format(format);
    table.add_row(row!["NAME", "VERSION", "KIND", "DESCRIPTION"]);

    for package in packages {
        let name = pad_str(&package.name, 20, Alignment::Left, Some(".."));
        let version = pad_str(&package.version, 10, Alignment::Left, Some(".."));
        let kind = pad_str(&package.kind, 10, Alignment::Left, Some(".."));
        let description = &package.description.unwrap_or_default();
        let description = pad_str(description, 50, Alignment::Left, Some(".."));

        table.add_row(row![name, version, kind, description]);
    }

    table.printstd();

    Ok(())
}

///
///
///
pub async fn get_package_index() -> Result<PackageIndex> {
    let packages_dir = packages::get_packages_dir();

    let packages: JValue = if packages_dir.exists() {
        let packages_dir = packages::get_packages_dir();
        if !packages_dir.exists() {
            return Ok(PackageIndex::empty());
        }

        let mut package_infos = Vec::<PackageInfo>::new();

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

                if let Ok(package_info) = PackageInfo::from_path(package_file) {
                    package_infos.push(package_info);
                }
            }
        }

        json!(package_infos)
    } else {
        let url = get_registry_endpoint(String::new())?;
        reqwest::get(url.as_str()).await?.json().await?
    };

    PackageIndex::from_value(packages)
}

///
///
///
pub async fn get_package_source(
    name: &String,
    version: &String,
    kind: &String,
) -> Result<PathBuf> {
    let package_dir = packages::get_package_dir(name, Some(version))?;
    let temp_dir = PathBuf::from("/tmp"); // TODO: get from OS

    let path = match kind.as_str() {
        "dsl" => {
            let instructions = package_dir.join("instructions.yml");
            if instructions.exists() {
                instructions
            } else {
                let instructions = temp_dir.join(format!("{}-{}-instructions.yml", name, version));
                if !instructions.exists() {
                    let url = get_registry_endpoint(format!("/{}/{}/source", name, version))?;
                    let mut source = reqwest::get(&url).await?;

                    // Write package archive to temporary file
                    let mut source_file = File::create(&instructions)?;
                    while let Some(chunk) = source.chunk().await? {
                        source_file.write_all(&chunk)?;
                    }
                }

                instructions
            }
        }
        "cwl" | "ecu" | "oas" => {
            let image_file = package_dir.join("image.tar");
            if image_file.exists() {
                image_file
            } else {
                let archive_dir = temp_dir.join(format!("{}-{}-archive", name, version));
                fs::create_dir_all(&archive_dir)?;

                let image_file = archive_dir.join("image.tar");
                if !image_file.exists() {
                    let url = get_registry_endpoint(format!("/{}/{}/archive", name, version))?;
                    let mut archive = reqwest::get(&url).await?;

                    // Write package archive to temporary file
                    let archive_path = temp_dir.join(format!("{}-{}-archive.tar.gz", name, version));
                    let mut archive_file = File::create(&archive_path)?;
                    while let Some(chunk) = archive.chunk().await? {
                        archive_file.write_all(&chunk)?;
                    }

                    // Unpack
                    let archive_file = File::open(archive_path)?;
                    let mut archive = Archive::new(GzDecoder::new(archive_file));
                    archive.unpack(&archive_dir)?;
                }

                image_file
            }
        }
        _ => unreachable!(),
    };

    Ok(path)
}

///
///
///
pub fn get_registry_endpoint(path: String) -> Result<String> {
    let config_file = utils::get_config_dir().join("registry.yml");
    let config = RegistryConfig::from_path(&config_file)
        .with_context(|| "No registry configuration found, please use `brane login` first.")?;

    Ok(format!("{}/packages{}", config.url, path))
}

///
///
///
pub async fn unpublish(
    name: String,
    version: String,
    force: bool,
) -> Result<()> {
    let url = get_registry_endpoint(format!("/{}/{}", name, version))?;

    let client = Client::new();
    let package = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to reach registry at: {}", url))?;

    if !package.status().is_success() {
        println!(
            "Remote registry doesn't contain version {} of package {}.",
            style(&version).bold().cyan(),
            style(&name).bold().cyan(),
        );

        return Ok(());
    }

    // Ask for permission, if --force is not provided
    if !force {
        println!("Do you want to remove the following version(s)?");
        println!("- {}", version);

        // Abort, if not approved
        if !Confirm::new().interact()? {
            return Ok(());
        }
    }

    client
        .delete(&url)
        .send()
        .await
        .with_context(|| "Failed to delete '{}' with version '{}' at remote registry.")?;

    Ok(())
}

use crate::packages;
use crate::utils;
use anyhow::Result;
use brane_dsl::indexes::PackageIndex;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest::{self, multipart::Form, multipart::Part, Body, Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JValue};
use specifications::package::PackageInfo;
use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use tar::Archive;
use std::path::PathBuf;
use tokio::fs::File as TokioFile;
use tokio_util::codec::{BytesCodec, FramedRead};

lazy_static! {
    static ref API_HOST: String = {
        env::var("API_HOST").unwrap_or_else(|_| String::from("brane-api:8080"))
    };
}

///
///
///
pub fn login(
    _host: String,
    _username: String,
) -> Result<()> {
    unimplemented!()
}

///
///
///
pub fn logout(_host: String) -> Result<()> {
    unimplemented!()
}

///
///
///
pub async fn pull(
    _name: String,
    _version: Option<String>,
) -> Result<()> {
    // let version = version.expect("please provide version");

    // let url = format!("http://{}/packages/{}/{}", API_HOST.as_str(), name, version);
    // let package: Result<PackageInfo, _> = reqwest::get(&url).await?.json().await;
    // if package.is_err() {
    //     println!("Cannot find version '{}' of package '{}'", version, name);
    //     return Ok(());
    // }

    // let url = format!("http://{}/packages/{}/{}/archive", API_HOST.as_str(), name, version);
    // let mut package_archive = reqwest::get(&url).await?;
    // let package = package.unwrap();

    // // Write package archive to temporary file
    // let temp_filepath = env::temp_dir().join(package.filename);
    // let mut temp_file = File::create(&temp_filepath)?;
    // while let Some(chunk) = package_archive.chunk().await? {
    //     temp_file.write_all(&chunk)?; // If causes bug, use .write(&chunk)
    // }

    // // Verify checksum
    // let checksum = utils::calculate_crc32(&temp_filepath)?;
    // if checksum != package.checksum as u32 {
    //     println!("Download failed, checksums don't match!");
    //     return Ok(());
    // }

    // // Unpack temporary file to target location
    // let archive_file = File::open(temp_filepath)?;
    // let package_dir = packages::get_package_dir(&name, Some(&version))?;
    // fs::create_dir_all(&package_dir)?;

    // let mut archive = Archive::new(GzDecoder::new(archive_file));
    // archive.unpack(&package_dir)?;

    println!("Unimplemented");

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

    // Create package tarball
    let gz = GzEncoder::new(&archive_file, Compression::fast());
    let mut tar = tar::Builder::new(gz);
    tar.append_dir_all(".", package_dir)?;
    tar.into_inner()?;

    // Calcualte checksum
    let checksum = utils::calculate_crc32(&archive_filepath)?;

    // Upload file
    let url = format!(
        "http://{}/packages/{}/{}?checksum={}",
        API_HOST.as_str(), name, version, checksum
    );
    let request = Client::new().request(Method::POST, &url);

    let file = TokioFile::open(&archive_filepath).await?;
    let reader = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

    let mut form = Form::new();
    form = form.part("file", Part::stream(reader).file_name(archive_filename));

    let request = request.multipart(form);
    let response = request.send().await?;

    println!("{:?}", response.text().await?);

    Ok(())
}

///
///
///
pub async fn search(term: String) -> Result<()> {
    let url = format!("http://{}/packages?t={}", API_HOST.as_str(), term);
    let packages: Vec<PackageInfo> = reqwest::get(&url).await?.json().await?;

    for package in packages {
        println!("{}", package.name);
    }

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
        let url = format!("http://{}/packages", API_HOST.as_str());
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
    kind: &String
) -> Result<PathBuf> {
    let package_dir = packages::get_package_dir(name, Some(version))?;
    let temp_dir = PathBuf::from("/tmp"); // TODO: get from OS

    let path = match kind.as_str() {
        "cwl" => {
            let cwl_file = package_dir.join("document.cwl");
            if cwl_file.exists() {
                cwl_file
            } else {
                let cwl_file = temp_dir.join(format!("{}-{}-document.cwl", name, version));
                if !cwl_file.exists() {
                    let url = format!("http://{}/packages/{}/{}/source", API_HOST.as_str(), name, version);
                    let mut source = reqwest::get(&url).await?;

                    // Write package archive to temporary file
                    let mut source_file = File::create(&cwl_file)?;
                    while let Some(chunk) = source.chunk().await? {
                        source_file.write_all(&chunk)?;
                    }
                }

                cwl_file
            }
        },
        "dsl" => {
            let instructions = package_dir.join("instructions.yml");
            if instructions.exists() {
                instructions
            } else {
                let instructions = temp_dir.join(format!("{}-{}-instructions.yml", name, version));
                if !instructions.exists() {
                    let url = format!("http://{}/packages/{}/{}/source", API_HOST.as_str(), name, version);
                    let mut source = reqwest::get(&url).await?;

                    // Write package archive to temporary file
                    let mut source_file = File::create(&instructions)?;
                    while let Some(chunk) = source.chunk().await? {
                        source_file.write_all(&chunk)?;
                    }
                }

                instructions
            }
        },
        "ecu" => {
            let image_file = package_dir.join("image.tar");
            if false && image_file.exists() {
                image_file
            } else {
                let archive_dir = temp_dir.join(format!("{}-{}-archive", name, version));
                fs::create_dir_all(&archive_dir)?;

                let image_file = archive_dir.join("image.tar");
                if !image_file.exists() {
                    let url = format!("http://{}/packages/{}/{}/archive", API_HOST.as_str(), name, version);
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
        },
        "oas" => {
            let oas_file = package_dir.join("document.yml");
            if oas_file.exists() {
                oas_file
            } else {
                let oas_file = temp_dir.join(format!("{}-{}-document.yml", name, version));
                if !oas_file.exists() {
                    let url = format!("http://{}/packages/{}/{}/source", API_HOST.as_str(), name, version);
                    let mut source = reqwest::get(&url).await?;

                    // Write package archive to temporary file
                    let mut source_file = File::create(&oas_file)?;
                    while let Some(chunk) = source.chunk().await? {
                        source_file.write_all(&chunk)?;
                    }
                }

                oas_file
            }
        },
        _ => unreachable!()
    };

    Ok(path)
}

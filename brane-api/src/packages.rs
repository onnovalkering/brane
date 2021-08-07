use anyhow::{Result, Context as _};
use bytes::Bytes;
use flate2::read::GzDecoder;
use scylla::Session;
use specifications::package::PackageInfo;
use tar::Archive;
use tokio::process::Command;
use warp::{Rejection, Reply, http::{HeaderValue, StatusCode}, hyper::HeaderMap};
use std::sync::Arc;
use crate::Context;

///
///
///
pub async fn ensure_db_table(scylla: &Session) -> Result<()> {
    let query = r#"
        CREATE TABLE IF NOT EXISTS brane.packages (
              created timestamp
            , description text
            , detached boolean
            , functions_as_json text
            , id uuid
            , kind text
            , name text
            , owners list<text>
            , types_as_json text
            , version text
            , PRIMARY KEY (name, version)
        );
    "#;

    scylla
        .query(query, &[])
        .await
        .map(|_| Ok(()))
        .map_err(|e| anyhow!("{:?}", e))
        .context("Failed to create 'brane.packages' table.")?
}

///
///
///
pub async fn upload(
    headers: HeaderMap<HeaderValue>, 
    package_archive: Bytes,
    context: Context,
) -> Result<impl Reply, Rejection> {
    println!("{:?}", headers);

    // Create temporary file for uploaded package archive.
    let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
        error!("An error occured while create a temporary file: {}", e);
        warp::reject::reject()
    })?;

    let (temp_file, temp_file_path) = temp_file.keep().unwrap();
    println!("{}", temp_file_path.display());

    tokio::fs::write(&temp_file_path, package_archive).await.map_err(|e| {
        error!("An error occured while writing to a temporary file: {}", e);
        warp::reject::reject()
    })?;

    // Unpack package archive to temporary working directory.
    let temp_dir = tempfile::tempdir().map_err(|e| {
        error!("An error occured while create a temporary directory: {}", e);
        warp::reject::reject()
    })?;

    let tar = GzDecoder::new(&temp_file);
    Archive::new(tar).unpack(&temp_dir).map_err(|e| {
        error!("An error occured while extracting a package archive: {}", e);
        warp::reject::reject()
    })?;

    // Parse package information
    let package_info_file = temp_dir.path().join("package.yml");
    let package_info = PackageInfo::from_path(package_info_file).map_err(|e| {
        error!("An error occured while parsing package information: {}", e);
        warp::reject::reject()
    })?;

    let name = &package_info.name;
    let version = &package_info.version;
    
    match package_info.kind.as_str() {
        "cwl" => {
            todo!();
        }
        "dsl" => {
            todo!();
        }
        "ecu" | "oas" => {
            // In the case of a container package, store image in Docker registry
            // TODO: make seperate function
            let image_tar = temp_dir.path().join("image.tar");
            if image_tar.exists() {
                let image_tar = image_tar.into_os_string().into_string().unwrap();
                let image_label = format!("{}:{}", name, version);

                let push = Command::new("skopeo")
                        .arg("copy")
                        .arg("--dest-tls-verify=false")
                        .arg(format!("docker-archive:{}", image_tar))
                        .arg(format!("docker://{}/library/{}", context.registry, image_label))
                        .status();

                push.await.map_err(|e| {
                    error!("An error occured while pushing the image to the registry: {}", e);
                    warp::reject::reject()
                })?;
            }
        }
        _ => unreachable!(),
    }

    insert_package_into_scyla(&package_info, &context.scylla).await.map_err(|e| {
        error!("An error occured while pushing the image to the registry: {}", e);
        warp::reject::reject()
    })?;

    debug!("{:?}", package_info);

    Ok(StatusCode::OK)
}

///
///
///
async fn insert_package_into_scyla(package: &PackageInfo, scylla: &Arc<Session>) -> Result<()> {
    let query = scylla.prepare(r#"
        INSERT INTO brane.packages (
              created
            , description
            , detached
            , functions_as_json
            , id
            , kind
            , name
            , owners
            , types_as_json
            , version
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#).await?;

    let functions_as_json = serde_json::to_string(&package.functions)?;
    let types_as_json = serde_json::to_string(&package.types)?;

    let values = (
        package.created.timestamp_millis(),
        &package.description,
        package.detached,
        functions_as_json,
        package.id,
        &package.kind,
        &package.name,
        &package.owners,
        types_as_json,
        &package.version,
    );

    scylla
        .execute(&query, values)
        .await
        .with_context(|| format!("Failed to insert package into database: {:?}", package))?;

    Ok(())
}
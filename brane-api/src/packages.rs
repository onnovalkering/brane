use crate::Context;
use anyhow::{Context as _, Result};
use bytes::Bytes;
use flate2::read::GzDecoder;
use scylla::cql_to_rust::FromCqlVal;
use scylla::macros::{FromUserType, IntoUserType};
use scylla::Session;
use specifications::package::PackageInfo;
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use tar::Archive;
use tokio::fs::File as TokioFile;
use tokio::process::Command;
use tokio_util::codec::{BytesCodec, FramedRead};
use uuid::Uuid;
use warp::http::HeaderValue;
use warp::hyper::Body;
use warp::reply::Response;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Clone, IntoUserType, FromUserType)]
pub struct PackageUdt {
    pub created: i64,
    pub description: String,
    pub detached: bool,
    pub functions_as_json: String,
    pub id: Uuid,
    pub kind: String,
    pub name: String,
    pub owners: Vec<String>,
    pub types_as_json: String,
    pub version: String,
}

impl TryFrom<PackageInfo> for PackageUdt {
    type Error = anyhow::Error;

    fn try_from(package: PackageInfo) -> Result<Self> {
        let functions_as_json = serde_json::to_string(&package.functions.clone().unwrap_or_default())?;
        let types_as_json = serde_json::to_string(&package.types.clone().unwrap_or_default())?;

        Ok(Self {
            created: package.created.timestamp_millis(),
            description: package.description,
            detached: package.detached,
            functions_as_json,
            id: package.id,
            kind: package.kind,
            name: package.name,
            owners: package.owners,
            types_as_json,
            version: package.version,
        })
    }
}

///
///
///
pub async fn ensure_db_table(scylla: &Session) -> Result<()> {
    scylla
        .query(
            "CREATE TYPE IF NOT EXISTS brane.package (
                  created bigint
                , description text
                , detached boolean
                , functions_as_json text
                , id uuid
                , kind text
                , name text
                , owners list<text>
                , types_as_json text
                , version text
            )",
            &[],
        )
        .await
        .context("Failed to create 'brane.package' type.")?;

    scylla
        .query(
            "CREATE TABLE IF NOT EXISTS brane.packages (
                  name text
                , package frozen<package>
                , version text
                , PRIMARY KEY (name, version)
            )",
            &[],
        )
        .await
        .context("Failed to create 'brane.packages' table.")?;

    Ok(())
}

///
///
///
async fn insert_package_into_db(
    package: &PackageInfo,
    scylla: &Arc<Session>,
) -> Result<()> {
    let package: PackageUdt = package.clone().try_into()?;

    scylla
        .query(
            "INSERT INTO brane.packages (
                  name
                , package
                , version
            ) VALUES(?, ?, ?)
            ",
            (&package.name, &package, &package.version),
        )
        .await
        .context("Failed to insert package into database.")?;

    Ok(())
}

///
///
///
pub async fn download(
    name: String,
    version: String,
    context: Context,
) -> Result<impl Reply, Rejection> {
    let image_tar = tempfile::NamedTempFile::new().expect("Failed to create temporary file.");
    let image_tar_str = image_tar.path().to_string_lossy().to_string();
    let image_label = format!("{}:{}", name, version);

    let pull = Command::new("skopeo")
        .arg("copy")
        .arg("--src-tls-verify=false")
        .arg(format!("docker://{}/library/{}", context.registry, image_label))
        .arg(format!("docker-archive:{}", &image_tar_str))
        .status();

    pull.await.map_err(|e| {
        error!("An error occured while pulling the image from the registry: {}", e);
        warp::reject::reject()
    })?;

    let file = TokioFile::open(&image_tar)
        .await
        .expect("Failed to open/read temporary file.");
    let file = FramedRead::new(file, BytesCodec::new());

    let mut response = Response::new(Body::wrap_stream(file));

    response.headers_mut().insert(
        "Content-Disposition",
        HeaderValue::from_static("attachment; filename=image.tar"),
    );

    response.headers_mut().insert(
        "Content-Length",
        HeaderValue::from(image_tar.path().metadata().unwrap().len()),
    );

    Ok(response)
}

///
///
///
pub async fn upload(
    package_archive: Bytes,
    context: Context,
) -> Result<impl Reply, Rejection> {
    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temporary file.");

    tokio::fs::write(&temp_file.path(), package_archive)
        .await
        .map_err(|e| {
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

    insert_package_into_db(&package_info, &context.scylla)
        .await
        .map_err(|e| {
            error!("An error occured while pushing the image to the registry: {}", e);
            warp::reject::reject()
        })?;

    debug!("{:?}", package_info);

    Ok(StatusCode::OK)
}

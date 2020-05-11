use crate::models::{Config, NewPackage, Package};
use crate::schema::{self, packages::dsl as db};
use actix_multipart::Multipart;
use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};
use crc32fast::Hasher;
use diesel::prelude::*;
use diesel::{r2d2, r2d2::ConnectionManager};
use flate2::read::GzDecoder;
use futures::{StreamExt, TryStreamExt};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use specifications::package::PackageInfo;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
type FResult<T> = Result<T, failure::Error>;
type Map<T> = std::collections::HashMap<String, T>;
type Query = web::Query<Map<String>>;

const MSG_NO_CHECKSUM: &str = "Checksum not provided.";
const MSG_NAME_CONFLICT: &str = "Package with the same name and version already exists.";
const MSG_NO_PACKAGE_INFO: &str = "Package doesn't contain a (valid) package.yml file.";
const MSG_INFO_MISMATCH: &str = "Package information doesn't match the HTTP request.";

///
///
///
pub fn scope() -> Scope {
    web::scope("/packages")
        .route("", web::get().to(get_packages))
        .route("/{name}", web::get().to(get_package))
        .route("/{name}/{version}", web::post().to(upload_package))
        .route("/{name}/{version}", web::get().to(get_package_version))
        .route("/{name}/{version}", web::delete().to(delete_package_version))
        .route("/{name}/{version}/archive", web::get().to(download_package_archive))
}

///
///
///
async fn get_packages(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    web::Query(query): Query,
) -> HttpResponse {
    let conn = pool.get().expect("Couldn't get connection from db pool.");
    let term = query.get("t").map(|t| String::from(t)).unwrap_or(String::new());

    let packages = db::packages
        .filter(db::name.like(format!("%{}%", term)))
        .load::<Package>(&conn);

    if let Ok(packages) = packages {
        HttpResponse::Ok().json(packages)
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn upload_package(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    web::Query(query): Query,
    mut payload: Multipart,
) -> HttpResponse {
    let conn = pool.get().expect("Couldn't get connection from db pool.");
    let config = config.get_ref().clone();
    let packages_dir = config.packages_dir;
    let temporary_dir = config.temporary_dir;
    let name = path.0.clone();
    let version = path.1.clone();

    // Validate request
    let checksum = if let Some(checksum) = query.get("checksum") {
        checksum.parse::<u32>().unwrap()
    } else {
        return upload_badrequest(MSG_NO_CHECKSUM, temporary_dir, None);
    };

    // TODO: validate name
    // TODO: validate version

    let package_exists = diesel::select(diesel::dsl::exists(
        db::packages.filter(db::name.eq(&name)).filter(db::version.eq(&version)),
    ))
    .get_result(&conn)
    .unwrap_or(true);

    if package_exists {
        return upload_badrequest(MSG_NAME_CONFLICT, temporary_dir, None);
    }

    // Generate identifier unique to this upload
    let upload_id: String = thread_rng().sample_iter(&Alphanumeric).take(12).collect();

    // Write uploaded file to temporary dir
    let mut crc32_hasher = Hasher::new();
    let temp_filename = format!("{}.tar.gz", &upload_id);
    let temp_filepath = temporary_dir.join(&temp_filename);
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filepath = temp_filepath.clone();

        // Filesystem operations are blocking, thus we use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await.unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            crc32_hasher.update(&data);

            f = web::block(move || f.write_all(&data).map(|_| f)).await.unwrap();
        }
    }

    // Check if file has been uploaded correctly
    let upload_checksum = crc32_hasher.finalize();
    if checksum != upload_checksum {
        let message = "Checksums don't match.";
        return upload_badrequest(message, temporary_dir, Some(upload_id));
    }

    // Unpack package to temporary working dir
    let temp_workdir = temporary_dir.join(&upload_id);
    let tar = GzDecoder::new(File::open(&temp_filepath).unwrap());
    Archive::new(tar).unpack(&temp_workdir).unwrap();

    // Parse package information
    let package_info_file = temp_workdir.join("package.yml");
    let new_package = if let Ok(info) = PackageInfo::from_path(package_info_file) {
        NewPackage::from_info(info, upload_checksum, temp_filename.clone())
    } else {
        return upload_badrequest(MSG_NO_PACKAGE_INFO, temporary_dir, Some(upload_id));
    };

    if new_package.name != name || new_package.version != version {
        return upload_badrequest(MSG_INFO_MISMATCH, temporary_dir, Some(upload_id));
    }

    // Store package information in database and the archive in the packages dir
    let result = web::block(move || {
        fs::copy(temp_filepath, packages_dir.join(temp_filename)).unwrap();
        upload_cleanup(temporary_dir, upload_id).unwrap();

        diesel::insert_into(schema::packages::table)
            .values(&new_package)
            .execute(&conn)
    }).await;

    if let Ok(_) = result {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
fn upload_badrequest(
    message: &str,
    temporary_dir: PathBuf,
    upload_id: Option<String>,
) -> HttpResponse {
    if let Some(upload_id) = upload_id {
        if let Err(_) = upload_cleanup(temporary_dir, upload_id) {
            return HttpResponse::InternalServerError().body("");
        }
    }

    HttpResponse::BadRequest().body(String::from(message))
}

///
///
///
fn upload_cleanup(
    temporary_dir: PathBuf,
    upload_id: String,
) -> FResult<()> {
    let temp_filepath = temporary_dir.join(format!("{}.tar.gz", &upload_id));
    let temp_workdir = temporary_dir.join(&upload_id);

    fs::remove_file(temp_filepath)?;
    fs::remove_dir_all(temp_workdir)?;

    Ok(())
}

///
///
///
async fn get_package(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect("Couldn't get connection from db pool.");
    let name = &path.0;

    let packages = db::packages.filter(db::name.eq(name)).load::<Package>(&conn);

    if let Ok(packages) = packages {
        if packages.len() > 0 {
            HttpResponse::Ok().json(packages)
        } else {
            HttpResponse::NotFound().body("")
        }
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_package_version(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let conn = pool.get().expect("Couldn't get connection from db pool.");
    let name = &path.0;
    let version = &path.1;

    let packages = db::packages
        .filter(db::name.eq(name))
        .filter(db::version.eq(version))
        .load::<Package>(&conn);

    if let Ok(packages) = packages {
        if packages.len() > 0 {
            HttpResponse::Ok().json(packages)
        } else {
            HttpResponse::NotFound().body("")
        }
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn delete_package_version(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let conn = pool.get().expect("Couldn't get connection from db pool.");
    let packages_dir = &config.get_ref().packages_dir;
    let name = &path.0;
    let version = &path.1;

    let package = db::packages
        .filter(db::name.eq(name))
        .filter(db::version.eq(version))
        .first::<Package>(&conn)
        .optional()
        .unwrap();

    if let None = package {
        return HttpResponse::NotFound().body("");
    }

    let package = package.unwrap();
    if let Err(_) = fs::remove_file(packages_dir.join(&package.filename)) {
        return HttpResponse::InternalServerError().body("Failed to delete package archive (tar.gz).");
    }

    if let Ok(_) = diesel::delete(&package).execute(&conn) {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn download_package_archive(
    _req: HttpRequest,
    _pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let name = &path.0;
    let version = &path.1;

    HttpResponse::NotImplemented().body(format!("Get {}:{}", name, version))
}

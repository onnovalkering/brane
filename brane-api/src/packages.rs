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
use tar::Archive;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
type Map<T> = std::collections::HashMap<String, T>;
type Query = web::Query<Map<String>>;

///
///
///
pub fn scope() -> Scope {
    web::scope("/packages")
        .route("", web::get().to(get_packages))
        .route("", web::post().to(upload_package))
        .route("/{name}", web::get().to(get_package))
        .route("/{name}/{version}", web::get().to(get_package_version))
        .route("/{name}/{version}/file", web::get().to(download_package_version))
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
    web::Query(query): Query,
    mut payload: Multipart,
) -> HttpResponse {
    let packages_dir = &config.get_ref().packges_dir;
    let temporary_dir = &config.get_ref().temporary_dir;

    // Generate identifier unique to this upload
    let upload_id: String = thread_rng().sample_iter(&Alphanumeric).take(12).collect();

    // Make sure a checksum is provided
    let checksum = if let Some(checksum) = query.get("checksum") {
        checksum.parse::<u32>().unwrap()
    } else {
        return HttpResponse::BadRequest().body("Checksum not provided.");
    };

    // Write uploaded file to temporary dir
    let mut crc32_hasher = Hasher::new();
    let temp_filename = format!("{}.tar.gz", upload_id);
    let temp_filepath = &temporary_dir.join(temp_filename.clone());
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
        return HttpResponse::BadRequest().body("Checksums don't match.");
    }

    // Unpack package to temporary working dir
    let temp_workdir = temporary_dir.join(upload_id);
    let tar = GzDecoder::new(File::open(temp_filepath).unwrap());
    Archive::new(tar).unpack(&temp_workdir).unwrap();

    let package_info = PackageInfo::from_path(temp_workdir.join("package.yml"));
    if package_info.is_err() {
        return HttpResponse::BadRequest().body("Package doesn't contain a (valid) package.yml file.");
    }

    // Store package information in database and tarball in packages dir
    fs::copy(temp_filepath, packages_dir.join(&temp_filename)).unwrap();

    let conn = pool.get().expect("Couldn't get connection from db pool.");
    let new_package = NewPackage::from_info(package_info.unwrap(), upload_checksum, temp_filename);
    let result = diesel::insert_into(schema::packages::table)
        .values(&new_package)
        .execute(&conn);


    if let Ok(_) = result {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_package(
    _req: HttpRequest,
    _pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let name = &path.0;

    HttpResponse::NotImplemented().body(format!("Get {}", name))
}

///
///
///
async fn get_package_version(
    _req: HttpRequest,
    _pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let name = &path.0;
    let version = &path.1;

    HttpResponse::NotImplemented().body(format!("Get {}:{}", name, version))
}

///
///
///
async fn download_package_version(
    _req: HttpRequest,
    _pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let name = &path.0;
    let version = &path.1;

    HttpResponse::NotImplemented().body(format!("Get {}:{}", name, version))
}

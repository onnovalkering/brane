use actix_web::Scope;
use actix_web::{web, HttpResponse};

///
///
///
pub fn scope() -> Scope {
    web::scope("/packages")
        .route("", web::get().to(get_packages))
        .route("", web::post().to(push_package))
        .route("/{name}", web::get().to(get_package))
        .route("/{name}/{version}", web::get().to(get_package_version))
}

///
///
///
async fn get_packages() -> HttpResponse {
    HttpResponse::NotImplemented().body("")
}

///
///
///
async fn push_package() -> HttpResponse {
    HttpResponse::NotImplemented().body("")
}

///
///
///
async fn get_package(path: web::Path<(String,)>) -> HttpResponse {
    let name = &path.0;

    HttpResponse::NotImplemented().body(format!("Get {}", name))
}

///
///
///
async fn get_package_version(path: web::Path<(String, String)>) -> HttpResponse {
    let name = &path.0;
    let version = &path.1;

    HttpResponse::NotImplemented().body(format!("Get {}:{}", name, version))
}

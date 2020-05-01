use actix_web::middleware;
use actix_web::{App, HttpServer};

mod packages;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath)
            .service(packages::scope())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

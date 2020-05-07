#[macro_use]
extern crate diesel;

use actix_web::middleware;
use actix_web::{App, HttpServer};
use diesel::SqliteConnection;
use diesel::{r2d2, r2d2::ConnectionManager};
use dotenv::dotenv;
use log::LevelFilter;
use std::env;
use structopt::StructOpt;

mod packages;

#[derive(StructOpt)]
#[structopt(name = "brane-api", about = "The Brane API service.")]
struct CLI {
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
    #[structopt(short = "o", long, help = "Host to bind", default_value = "127.0.0.1")]
    host: String,
    #[structopt(short, long, help = "Port to bind", default_value = "8080")]
    port: u16,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    let options = CLI::from_args();
    if options.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    // Create a database pool
    let database_url = env::var("DATABASE_URL").unwrap_or(String::from("db.sqlite"));
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    // Configure the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath)
            .data(pool.clone())
            .service(packages::scope())
    });

    let address = format!("{}:{}", options.host, options.port);
    server.bind(address)?.run().await
}

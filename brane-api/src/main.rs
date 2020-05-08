#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::middleware;
use actix_web::{App, HttpServer};
use diesel::SqliteConnection;
use diesel::{r2d2, r2d2::ConnectionManager};
use dotenv::dotenv;
use log::LevelFilter;
use std::env;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod models;
mod packages;
mod schema;

embed_migrations!();

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

    // Prepare the filesystem
    let packages_dir = env::var("PACKAGES_DIR").unwrap_or(String::from("./packages"));
    let temporary_dir = env::var("TEMPORARY_DIR").unwrap_or(String::from("./temporary"));
    fs::create_dir_all(&temporary_dir)?;
    fs::create_dir_all(&packages_dir)?;

    // Create a database pool
    let database_url = env::var("DATABASE_URL").unwrap_or(String::from("db.sqlite"));
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    // Run database migrations
    let conn = pool.get().expect("Couldn't get connection from db pool.");
    embedded_migrations::run(&conn).expect("Failed to run database migrations.");

    // Configure the HTTP server
    let config = models::Config {
        packges_dir: PathBuf::from(packages_dir),
        temporary_dir: PathBuf::from(temporary_dir),
    };
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath)
            .data(config.clone())
            .data(pool.clone())
            .service(packages::scope())
    });

    let address = format!("{}:{}", options.host, options.port);
    server.bind(address)?.run().await
}

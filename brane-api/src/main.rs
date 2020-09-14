#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;

use actix_web::middleware;
use actix_web::{App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use log::LevelFilter;
use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;
use std::env;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use redis::Client;

mod callback;
mod health;
mod invocations;
mod models;
mod packages;
mod schema;
mod sessions;

embed_migrations!();

const DEF_DATABASE_URL: &str = "postgres://postgres:postgres@postgres/postgres";
const DEF_DOCKER_HOST: &str = "registry:5000";
const DEF_KAFKA_BROKERS: &str = "kafka:9092";
const DEF_PACKAGES_DIR: &str = "./packages";
const DEF_REDIS_URL: &str = "redis://redis";
const DEF_TEMPORARY_DIR: &str = "./temporary";

#[derive(StructOpt)]
#[structopt(name = "brane-api", about = "The Brane API service.")]
struct CLI {
    #[structopt(short, long, help = "Enable debug mode", env = "DEBUG")]
    debug: bool,
    #[structopt(short = "o", long, help = "Host to bind", default_value = "127.0.0.1", env = "HOST")]
    host: String,
    #[structopt(short, long, help = "Port to bind", default_value = "8080", env = "PORT")]
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
    let packages_dir = env::var("PACKAGES_DIR").unwrap_or_else(|_| String::from(DEF_PACKAGES_DIR));
    let temporary_dir = env::var("TEMPORARY_DIR").unwrap_or_else(|_| String::from(DEF_TEMPORARY_DIR));
    fs::create_dir_all(&temporary_dir)?;
    fs::create_dir_all(&packages_dir)?;

    // Create a database pool
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| String::from(DEF_DATABASE_URL));
    let db_manager = ConnectionManager::<PgConnection>::new(db_url);
    let db_pool = r2d2::Pool::builder().build(db_manager).expect("Failed to create DB connection pool.");

    // Create a Redis connection pool
    let rd_url = env::var("REDIS_URL").unwrap_or_else(|_| String::from(DEF_REDIS_URL));
    let rd_manager = Client::open(rd_url).unwrap();
    let rd_pool = r2d2::Pool::builder().build(rd_manager).expect("Failed to create Redis connection pool.");

    // Run database migrations
    let conn = db_pool.get().expect("Couldn't get connection from db pool.");
    embedded_migrations::run(&conn).expect("Failed to run database migrations.");

    // Create Kafka producer
    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| String::from(DEF_KAFKA_BROKERS));
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Couldn't create a Kafka producer.");

    // Prepare configuration
    let docker_host = env::var("DOCKER_HOST").unwrap_or_else(|_| String::from(DEF_DOCKER_HOST));
    let config = models::Config {
        docker_host,
        packages_dir: PathBuf::from(packages_dir),
        temporary_dir: PathBuf::from(temporary_dir),
    };

    // Configure the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath)
            .data(config.clone())
            .data(db_pool.clone())
            .data(rd_pool.clone())
            .data(producer.clone())
            .service(callback::scope())
            .service(health::scope())
            .service(invocations::scope())
            .service(packages::scope())
            .service(sessions::scope())
    });

    let address = format!("{}:{}", options.host, options.port);
    server.bind(address)?.run().await
}

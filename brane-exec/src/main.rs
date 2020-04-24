#[macro_use]
extern crate log;

use actix_web::{post, web::Json, App, HttpResponse, HttpServer};
use executor::kubernetes;
use executor::{ExecuteRequest, ExecuteResponse};
use log::LevelFilter;
use std::env;

#[post("/execute")]
async fn execute(data: Json<ExecuteRequest>) -> HttpResponse {
    let request = data.into_inner();

    let result = match request.target.as_str() {
        "k8s" => kubernetes::schedule(request).await,
        "hpc" => unimplemented!(),
        "sys" => unimplemented!(),
        _ => unreachable!(),
    };

    let response = match result {
        Ok(identifier) => ExecuteResponse::ExecuteToken { identifier },
        Err(error) => {
            warn!("Failed to schedule:\n\n{}\n", error);

            let message = String::from("Failed!");
            ExecuteResponse::ExecuteError { message }
        }
    };

    HttpResponse::Ok().json(response)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if env::var_os("DEBUG").is_some() {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }

    HttpServer::new(|| App::new().service(execute))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

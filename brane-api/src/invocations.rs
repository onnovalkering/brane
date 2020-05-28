use crate::models::{Invocation, NewInvocation};
use crate::schema::{self, invocations::dsl as db};
use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use diesel::{r2d2, r2d2::ConnectionManager};
use futures::*;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Deserialize;
use specifications::common::Argument;
use specifications::instructions::Instruction;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type Map<T> = std::collections::HashMap<String, T>;

const MSG_NO_DB_CONNECTION: &str = "Couldn't get connection from db pool.";
const MSG_ONLY_STOPPED_DELETED: &str = "Only stopped invocations can be deleted.";
const MSG_ONLY_HALTED_RESUMED: &str = "Only halted invocations can be halted.";
const MSG_ONLY_RUNNING_STOPPED: &str = "Only running invocations can be stopped.";
const MSG_ONLY_RUNNING_SUSPENED: &str = "Only running invocations can be suspended.";

const TOPIC_CONTROL: &str = "control";

///
///
///
pub fn scope() -> Scope {
    web::scope("/invocations")
        .route("", web::post().to(create_invocation))
        .route("", web::get().to(get_invocations))
        .route("/{uuid}", web::get().to(get_invocation))
        .route("/{uuid}", web::delete().to(delete_invocation))
        .route("/{uuid}/resume", web::post().to(resume_invocation))
        .route("/{uuid}/stop", web::post().to(stop_invocation))
        .route("/{uuid}/suspend", web::post().to(suspend_invocation))
}

#[derive(Deserialize)]
pub struct CreateInvocation {
    pub name: Option<String>,
    pub arguments: Map<Argument>,
    pub instructions: Vec<Instruction>,
}

///
///
///
async fn create_invocation(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    producer: web::Data<FutureProducer>,
    json: web::Json<CreateInvocation>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    // Store invocation information in database
    let new_invocation = NewInvocation::new(json.name.clone(), &json.arguments, &json.instructions).unwrap();
    let invocation = web::block(move || {
        diesel::insert_into(schema::invocations::table)
            .values(&new_invocation)
            .get_result::<Invocation>(&conn)
    })
    .await;

    // Something went wrong when creating the invocation.
    if invocation.is_err() {
        return HttpResponse::InternalServerError().body("");
    }

    // Send control message (fire-and-forget) to announce the new invocation
    let invocation = invocation.unwrap();
    let delivery = announce_status(&producer, invocation.id, &invocation.status).await;

    if delivery.is_ok() {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_invocations(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocations = db::invocations.load::<Invocation>(&conn);

    if let Ok(invocations) = invocations {
        HttpResponse::Ok().json(invocations)
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_invocation(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocations = web::block(move || db::invocations.filter(db::uuid.eq(&path.0)).load::<Invocation>(&conn)).await;

    if let Ok(invocations) = invocations {
        if invocations.len() == 1 {
            let invocation_info: &Invocation = invocations.first().unwrap();
            HttpResponse::Ok().json(invocation_info)
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
async fn delete_invocation(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocation = db::invocations
        .filter(db::uuid.eq(&path.0))
        .first::<Invocation>(&conn)
        .optional()
        .unwrap();

    if invocation.is_none() {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != "stopped" {
        return HttpResponse::BadRequest().body(MSG_ONLY_STOPPED_DELETED);
    }

    if diesel::delete(&invocation).execute(&conn).is_ok() {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn resume_invocation(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    producer: web::Data<FutureProducer>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocation = db::invocations
        .filter(db::uuid.eq(&path.0))
        .first::<Invocation>(&conn)
        .optional()
        .unwrap();

    if invocation.is_none() {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != "halted" {
        return HttpResponse::BadRequest().body(MSG_ONLY_HALTED_RESUMED);
    }

    let invocation = web::block(move || {
        diesel::update(&invocation)
            .set(db::status.eq("resuming"))
            .get_result::<Invocation>(&conn)
    })
    .await;

    // Send control message (fire-and-forget) to announce the new invocation
    let invocation = invocation.unwrap();
    let delivery = announce_status(&producer, invocation.id, &invocation.status).await;

    if delivery.is_ok() {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn stop_invocation(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    producer: web::Data<FutureProducer>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocation = db::invocations
        .filter(db::uuid.eq(&path.0))
        .first::<Invocation>(&conn)
        .optional()
        .unwrap();

    if invocation.is_none() {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != "running" {
        return HttpResponse::BadRequest().body(MSG_ONLY_RUNNING_STOPPED);
    }

    let invocation = web::block(move || {
        diesel::update(&invocation)
            .set(db::status.eq("stoping"))
            .get_result::<Invocation>(&conn)
    })
    .await;

    // Send control message (fire-and-forget) to announce the new invocation
    let invocation = invocation.unwrap();
    let delivery = announce_status(&producer, invocation.id, &invocation.status).await;

    if delivery.is_ok() {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn suspend_invocation(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    producer: web::Data<FutureProducer>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocation = db::invocations
        .filter(db::uuid.eq(&path.0))
        .first::<Invocation>(&conn)
        .optional()
        .unwrap();

    if invocation.is_none() {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != "running" {
        return HttpResponse::BadRequest().body(MSG_ONLY_RUNNING_SUSPENED);
    }

    let invocation = web::block(move || {
        diesel::update(&invocation)
            .set(db::status.eq("suspending"))
            .get_result::<Invocation>(&conn)
    })
    .await;

    // Send control message (fire-and-forget) to announce the new invocation
    let invocation = invocation.unwrap();
    let delivery = announce_status(&producer, invocation.id, &invocation.status).await;

    if delivery.is_ok() {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn announce_status(
    producer: &FutureProducer,
    invocation_id: i32,
    status: &str,
) -> Result<(), ()> {
    let message_key = format!("inv#{}", invocation_id);
    let message_payload = format!(r#"{{"status": "{}"}}"#, status);
    let message = FutureRecord::to(TOPIC_CONTROL)
        .payload(&message_payload)
        .key(&message_key);

    producer
        .send(message, 0)
        .map(|delivery| {
            let delivery = delivery.unwrap();

            match delivery {
                Ok(_) => {
                    info!(
                        "Announced that the status of invocation #{} is now '{}'.",
                        invocation_id, status
                    );
                    Ok(())
                }
                Err(error) => {
                    info!(
                        "Unable to announced that the status of invocation #{} is now '{}':\n{:#?}",
                        invocation_id, status, error
                    );
                    Err(())
                }
            }
        })
        .await
}

use crate::models::{Invocation, NewInvocation, Session};
use crate::schema::{self, invocations::dsl as db, sessions::dsl as s_db};
use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use futures::*;
use rdkafka::producer::{FutureProducer, FutureRecord};
use redis::Commands;
use serde::Deserialize;
use serde_json::json;
use specifications::instructions::Instruction;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type RedisPool = r2d2::Pool<redis::Client>;

const MSG_NO_DB_CONNECTION: &str = "Couldn't get connection from db pool.";
const MSG_NO_RD_CONNECTION: &str = "Couldn't get connection from rd pool.";
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
        .route("/{uuid}/status", web::get().to(get_invocation_status))
}

#[derive(Deserialize)]
pub struct CreateInvocation {
    pub session: String,
    pub name: Option<String>,
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
    let invocation = web::block(move || {
        let session = s_db::sessions
            .filter(s_db::uuid.eq(&json.session))
            .first::<Session>(&conn)
            .unwrap();
        // TODO: check if session is "created" or "idle", i.e. there is currently no other invocation active

        let new_invocation = NewInvocation::new(session.id, json.name.clone(), &json.instructions).unwrap();
        diesel::insert_into(schema::invocations::table)
            .values(&new_invocation)
            .get_result::<Invocation>(&conn)
    })
    .await;

    // Something went wrong while creating the invocation.
    if invocation.is_err() {
        return HttpResponse::InternalServerError().body("");
    }

    // Send control message (fire-and-forget) to announce the new invocation
    let invocation = invocation.unwrap();
    let delivery = trigger_event(&producer, invocation.id, &invocation.status).await;

    if delivery.is_ok() {
        HttpResponse::Ok().json(invocation)
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
    let delivery = trigger_event(&producer, invocation.id, &invocation.status).await;

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
    let delivery = trigger_event(&producer, invocation.id, &invocation.status).await;

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
    let delivery = trigger_event(&producer, invocation.id, &invocation.status).await;

    if delivery.is_ok() {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_invocation_status(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    db_pool: web::Data<DbPool>,
    rd_pool: web::Data<RedisPool>,
) -> HttpResponse {
    let db_conn = db_pool.get().expect(MSG_NO_DB_CONNECTION);
    let mut rd_conn = rd_pool.get().expect(MSG_NO_RD_CONNECTION);

    let invocation = web::block(move || {
        db::invocations
            .filter(db::uuid.eq(&path.0))
            .first::<Invocation>(&db_conn)
    })
    .await;
    if invocation.is_err() {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    let position: i32 = rd_conn
        .get(format!("inv-{}_cursor_position", invocation.id))
        .unwrap_or(0);
    let depth: i32 = rd_conn.get(format!("inv-{}_cursor_depth", invocation.id)).unwrap_or(0);

    println!("{}", format!("inv-{}_cursor_depth: {}", invocation.id, depth));

    let subpositions: Vec<i32> = if depth > 0 {
        (1..depth + 1)
            .map(|d| {
                let subposition: i32 = rd_conn
                    .get(format!("inv-{}_cursor_subposition_{}", invocation.id, d))
                    .unwrap();
                subposition
            })
            .collect()
    } else {
        vec![]
    };

    let status = json!({
        "invocation": invocation,
        "cursor": {
            "position": position,
            "depth": depth,
            "subpositions": subpositions,
        }
    });

    HttpResponse::Ok().json(status)
}

///
///
///
async fn trigger_event(
    producer: &FutureProducer,
    invocation_id: i32,
    event: &str,
) -> Result<(), ()> {
    let message_key = format!("inv-{}", invocation_id);
    let message_payload = format!(r#"{{"event": "{}"}}"#, event);
    let message = FutureRecord::to(TOPIC_CONTROL)
        .payload(&message_payload)
        .key(&message_key);

    producer
        .send(message, 0)
        .map(|delivery| {
            let delivery = delivery.unwrap();

            match delivery {
                Ok(_) => {
                    info!("Triggered '{}' event for invocation #{}.", event, invocation_id);
                    Ok(())
                }
                Err(error) => {
                    info!(
                        "Unable to trigger '{}' event for of invocation #{}:\n{:#?}",
                        event, invocation_id, error
                    );
                    Err(())
                }
            }
        })
        .await
}

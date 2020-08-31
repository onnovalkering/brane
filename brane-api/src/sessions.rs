use crate::models::{Session, NewSession};
use crate::schema::{self, sessions::dsl as db};
use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use diesel::{r2d2, r2d2::ConnectionManager};
use serde::Deserialize;
use specifications::common::Value;
use specifications::instructions::Instruction;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type Map<T> = std::collections::HashMap<String, T>;

const MSG_NO_DB_CONNECTION: &str = "Couldn't get connection from db pool.";
const MSG_ONLY_STOPPED_DELETED: &str = "Only stopped sessions can be deleted.";

///
///
///
pub fn scope() -> Scope {
    web::scope("/sessions")
        .route("", web::post().to(create_session))
        .route("", web::get().to(get_sessions))
        .route("/{uuid}", web::get().to(get_session))
        .route("/{uuid}", web::delete().to(delete_session))
}

#[derive(Deserialize)]
pub struct CreateSession {
    pub name: Option<String>,
    pub arguments: Map<Value>,
    pub instructions: Vec<Instruction>,
}

///
///
///
async fn create_session(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
) -> HttpResponse { 
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    // Store invocation information in database
    let new_session = NewSession::new().unwrap();
    let session = web::block(move || {
        diesel::insert_into(schema::sessions::table)
            .values(&new_session)
            .get_result::<Session>(&conn)
    })
    .await;

    if session.is_ok() {
        HttpResponse::Ok().json(session.unwrap())
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_sessions(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let sessions = db::sessions.load::<Session>(&conn);

    if let Ok(sessions) = sessions {
        HttpResponse::Ok().json(sessions)
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_session(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let sessions = web::block(move || db::sessions.filter(db::uuid.eq(&path.0)).load::<Session>(&conn)).await;

    if let Ok(sessions) = sessions {
        if sessions.len() == 1 {
            let session: &Session = sessions.first().unwrap();
            HttpResponse::Ok().json(session)
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
async fn delete_session(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let session = db::sessions
        .filter(db::uuid.eq(&path.0))
        .first::<Session>(&conn)
        .optional()
        .unwrap();

    if session.is_none() {
        return HttpResponse::NotFound().body("");
    }

    let session = session.unwrap();
    if session.status != "stopped" {
        return HttpResponse::BadRequest().body(MSG_ONLY_STOPPED_DELETED);
    }

    if diesel::delete(&session).execute(&conn).is_ok() {
        HttpResponse::Ok().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}
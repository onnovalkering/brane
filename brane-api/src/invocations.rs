use crate::models::{Invocation, NewInvocation};
use crate::schema::{self, invocations::dsl as db};
use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use diesel::{r2d2, r2d2::ConnectionManager};
use serde::Deserialize;
use specifications::common::Argument;
use specifications::instructions::Instruction;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
type Map<T> = std::collections::HashMap<String, T>;

const MSG_NO_DB_CONNECTION: &str = "Couldn't get connection from db pool.";
const MSG_ONLY_STOPPED_DELETED: &str = "Only stopped invocations can be deleted.";
const MSG_ONLY_HALTED_RESUMED: &str = "Only halted invocations can be halted.";
const MSG_ONLY_RUNNING_STOPPED: &str = "Only running invocations can be stopped.";
const MSG_ONLY_RUNNING_SUSPENED: &str = "Only running invocations can be suspended.";

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
    json: web::Json<CreateInvocation>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    // Store invocation information in database
    let new_invocation = NewInvocation::new(json.name.clone(), &json.arguments, &json.instructions).unwrap();
    let result = web::block(move || {
        diesel::insert_into(schema::invocations::table)
            .values(&new_invocation)
            .execute(&conn)
    })
    .await;

    if let Ok(_) = result {
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

    let invocations = web::block(move || {
        db::invocations.filter(db::uuid.eq(&path.0)).load::<Invocation>(&conn)
    })
    .await;

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

    if let None = invocation {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != String::from("stopped") {
        return HttpResponse::BadRequest().body(MSG_ONLY_STOPPED_DELETED);
    }

    if let Ok(_) = diesel::delete(&invocation).execute(&conn) {
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
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocation = db::invocations
        .filter(db::uuid.eq(&path.0))
        .first::<Invocation>(&conn)
        .optional()
        .unwrap();

    if let None = invocation {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != String::from("halted") {
        return HttpResponse::BadRequest().body(MSG_ONLY_HALTED_RESUMED);
    }

    // TODO: resume

    let operation = web::block(move || {
        diesel::update(&invocation)
            .set(db::status.eq("resuming"))
            .execute(&conn)
    })
    .await;

    if let Ok(_) = operation {
        HttpResponse::Accepted().body("")
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
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocation = db::invocations
        .filter(db::uuid.eq(&path.0))
        .first::<Invocation>(&conn)
        .optional()
        .unwrap();

    if let None = invocation {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != String::from("running") {
        return HttpResponse::BadRequest().body(MSG_ONLY_RUNNING_STOPPED);
    }

    let operation = web::block(move || {
        diesel::update(&invocation)
            .set(db::status.eq("stoping"))
            .execute(&conn)
    })
    .await;

    if let Ok(_) = operation {
        HttpResponse::Accepted().body("")
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
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let invocation = db::invocations
        .filter(db::uuid.eq(&path.0))
        .first::<Invocation>(&conn)
        .optional()
        .unwrap();

    if let None = invocation {
        return HttpResponse::NotFound().body("");
    }

    let invocation = invocation.unwrap();
    if invocation.status != String::from("running") {
        return HttpResponse::BadRequest().body(MSG_ONLY_RUNNING_SUSPENED);
    }

    // TODO: halt

    let operation = web::block(move || {
        diesel::update(&invocation)
            .set(db::status.eq("suspending"))
            .execute(&conn)
    })
    .await;

    if let Ok(_) = operation {
        HttpResponse::Accepted().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

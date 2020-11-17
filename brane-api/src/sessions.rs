use anyhow::Result;
use crate::models::{Invocation, NewSession, NewVariable, Session, Variable};
use crate::schema::{self, invocations::dsl as inv_db, sessions::dsl as db, variables::dsl as var_db};
use actix_files::NamedFile;
use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use diesel::{r2d2, r2d2::ConnectionManager};
use serde::Deserialize;
use specifications::common::Value;
use std::path::PathBuf;
use url::Url;
use std::env;
use std::process::Command;
use std::fs;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type Map<T> = std::collections::HashMap<String, T>;

const MSG_NO_DB_CONNECTION: &str = "Couldn't get connection from db pool.";
const MSG_ONLY_STOPPED_DELETED: &str = "Only stopped sessions can be deleted.";

lazy_static! {
    static ref SYSTEM: String = env::var("SYSTEM").unwrap_or_else(|_| String::from("local"));
    static ref HOSTNAME: String = env::var("HPC_HOSTNAME").unwrap_or_else(|_| String::from("slurm"));
    static ref PORT: String = env::var("HPC_PORT").unwrap_or_else(|_| String::from("22"));
    static ref FILE_SYTEM: String = env::var("HPC_FILESYSTEM").unwrap_or_else(|_| String::from("sftp"));
    static ref XENON: String = env::var("HPC_XENON").unwrap_or_else(|_| String::from("localhost:50051"));
    static ref DATA_DIR: String = env::var("HPC_DATA_DIR").unwrap_or_else(|_| String::from("/home/xenon/"));

    // TODO: fetch credentials from vault (requires some refactoring to avoid circular dependencies).
    static ref USERNAME: String = env::var("HPC_USERNAME").unwrap_or_else(|_| String::from("xenon"));
    static ref PASSWORD: String = env::var("HPC_PASSWORD").unwrap_or_else(|_| String::from("javagat"));
}

///
///
///
pub fn scope() -> Scope {
    web::scope("/sessions")
        .route("", web::post().to(create_session))
        .route("", web::get().to(get_sessions))
        .route("/{uuid}", web::get().to(get_session))
        .route("/{uuid}", web::delete().to(delete_session))
        .route("/{uuid}/invocations", web::get().to(get_session_invocations))
        .route("/{uuid}/files/{variable}", web::get().to(get_session_file))
        .route("/{uuid}/variables", web::get().to(get_session_variables))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSession {
    pub arguments: Option<Map<Value>>,
    pub invocation_id: Option<i32>,
}

///
///
///
async fn create_session(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    json: web::Json<CreateSession>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    // Store session information in database
    let new_session = NewSession::new(json.invocation_id).unwrap();
    let session = diesel::insert_into(schema::sessions::table)
        .values(&new_session)
        .get_result::<Session>(&conn);

    if let Ok(session) = session {
        // Store arguments as session variables, if any
        if let Some(arguments) = &json.arguments {
            for (key, value) in arguments.iter() {
                let value_json = serde_json::to_string(value).unwrap();
                let new_variable = NewVariable::new(
                    session.id,
                    key.clone(),
                    value.data_type().to_string(),
                    Some(value_json.clone()),
                )
                .unwrap();

                diesel::insert_into(var_db::variables)
                    .values(&new_variable)
                    .execute(&conn)
                    .unwrap();
            }
        }

        HttpResponse::Ok().json(session)
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

    let session = web::block(move || db::sessions.filter(db::uuid.eq(&path.0)).first::<Session>(&conn)).await;

    if let Ok(session) = session {
        HttpResponse::Ok().json(session)
    } else {
        HttpResponse::NotFound().body("")
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InvocationsFilter {
    status: Option<String>,
}

///
///
///
async fn get_session_invocations(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
    filter: web::Query<InvocationsFilter>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let session = db::sessions.filter(db::uuid.eq(&path.0)).first::<Session>(&conn);
    if session.is_err() {
        return HttpResponse::NotFound().body("");
    }

    let session = session.unwrap();
    let invocations = if let Some(status) = &filter.status {
        Invocation::belonging_to(&session)
            .filter(inv_db::status.eq(status))
            .load::<Invocation>(&conn)
    } else {
        Invocation::belonging_to(&session).load::<Invocation>(&conn)
    };

    if let Ok(invocations) = invocations {
        HttpResponse::Ok().json(invocations)
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn get_session_file(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let session = db::sessions.filter(db::uuid.eq(&path.0)).first::<Session>(&conn);
    if session.is_err() {
        return HttpResponse::NotFound().body("Session not found.");
    }

    let variable_name = path.1.clone();
    let variable = if variable_name.contains('.') {
        let segments: Vec<_> = variable_name.split('.').collect();

        let object = Variable::belonging_to(&session.unwrap())
            .filter(var_db::name.eq(&segments[0]))
            .first::<Variable>(&conn);

        if let Ok(object) = object {
            let object: Value = serde_json::from_str(&object.content_json.unwrap()).unwrap();
            if let Value::Struct { properties, .. } = object {
                if segments[1].contains('[') {
                    let sub_segments: Vec<_> = segments[1].split('[').collect();
                    if let Some(value) = properties.get(sub_segments[0]) {
                        let index: usize = sub_segments[1].strip_suffix(']').unwrap().to_string().parse().unwrap();
                        if let Value::Array { entries, .. } = value {
                            entries.get(index).unwrap().clone()
                        } else {
                            return HttpResponse::BadRequest().body("Variable is not an array.");
                        }
                    } else {
                        return HttpResponse::NotFound().body("Variable not found.");
                    }
                } else {
                    if let Some(value) = properties.get(segments[1]) {
                        value.clone()
                    } else {
                        return HttpResponse::NotFound().body("Variable not found.");
                    }
                }
            } else {
                return HttpResponse::BadRequest().body("Variable is not a object.");
            }
        } else {
            return HttpResponse::NotFound().body("Variable not found.");
        }
    } else {
        let variable = Variable::belonging_to(&session.unwrap())
            .filter(var_db::name.eq(&path.1))
            .first::<Variable>(&conn);

        if let Ok(variable) = variable {
            let variable: Value = serde_json::from_str(&variable.content_json.unwrap()).unwrap();
            variable
        } else {
            return HttpResponse::NotFound().body("Variable not found. 3");
        }
    };

    if variable.data_type() != "File" {
        return HttpResponse::BadRequest().body("Variable is not of type 'File'.");
    }

    if let Value::Struct { properties, .. } = variable {
        let url = properties.get("url").expect("Missing `url` property on File.");
        let path = to_local_file(Url::parse(&url.as_string().unwrap()).unwrap()).await.unwrap();
        if !path.exists() {
            return HttpResponse::InternalServerError().body("");
        }

        let file = NamedFile::open(path).unwrap();
        return file.into_response(&req).unwrap();
    } else {
        return HttpResponse::InternalServerError().body("Illegal internal state.");
    }
}

///
///
///
async fn to_local_file(url: Url) -> Result<PathBuf> {
    let original = PathBuf::from(&url.path());

    if SYSTEM.as_str() == "local" || SYSTEM.as_str() == "kubernetes" || SYSTEM.as_str() == "docker" {
        return Ok(original);
    }

    let path = original.to_string_lossy();
    let path = path.strip_prefix(DATA_DIR.as_str()).expect("Failed to strip DATA_DIR from path.");

    info!("{:?}", path);

    let path = PathBuf::from("/tmp/brane").join(path);
    if path.exists() {
        return Ok(path);
    }

    info!("{:?}", path.parent().unwrap());

    fs::create_dir_all(path.parent().unwrap())?;
    info!("Created dirs");

    let output = Command::new("sshpass")
        .args(vec!["-p", PASSWORD.as_str()])
        .args(vec!["scp", "-P", PORT.as_str()])
        .arg(format!("{}@{}:{}", USERNAME.as_str(), HOSTNAME.as_str(), url.path()))
        .arg(&path)
        .output()?;

    info!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    warn!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    ensure!(output.status.success(), "Failed to download!");

    Ok(path)
}

///
///
///
async fn get_session_variables(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let conn = pool.get().expect(MSG_NO_DB_CONNECTION);

    let sessions = db::sessions.filter(db::uuid.eq(&path.0)).load::<Session>(&conn);
    let variables = if let Ok(sessions) = sessions {
        if sessions.len() == 1 {
            let session: &Session = sessions.first().unwrap();
            Variable::belonging_to(session).load::<Variable>(&conn)
        } else {
            return HttpResponse::NotFound().body("");
        }
    } else {
        return HttpResponse::InternalServerError().body("");
    };

    if let Ok(variables) = variables {
        HttpResponse::Ok().json(variables)
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

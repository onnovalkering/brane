use anyhow::Result;
use serde_json::Value as JValue;
use diesel::prelude::*;
use chrono::Utc;
use brane_sys::local::LocalSystem;
use brane_vm::async_machine::{AsyncMachine, MachineResult};
use brane_vm::vault::InMemoryVault;
use diesel::pg::PgConnection;
use diesel::r2d2::{Pool, ConnectionManager};
use redis::Client;
use brane_vm::cursor::RedisCursor;
use brane_vm::environment::{Environment, RedisEnvironment};
use crate::schema::{invocations::dsl as inv_db, variables::dsl as var_db, sessions::dsl as ses_db};
use crate::models::{Invocation, Variable, Session, NewVariable};
use redis::AsyncCommands;
use specifications::common::Value;

type DbPool = Pool<ConnectionManager<PgConnection>>;
type Event = (String, String);

const MSG_NO_DB_CONNECTION: &str = "Couldn't get connection from db pool.";
const MSG_NO_RD_CONNECTION: &str = "Couldn't get connection from rd client.";

///
///
///
pub async fn handle(context: String, payload: JValue, db: &DbPool, rd: &Client) -> Result<Vec<Event>> {
    let event = payload["event"].as_str().expect("Payload doesn't contain 'event' property.");

    match event {
        "active" => on_active(context, payload, db, rd).await,
        "callback" => on_callback(context, payload, db, rd).await,
        "complete" => on_complete(context, payload, db, rd).await,
        "created" => on_created(context, payload, db, rd).await,
        "ready" => on_ready(context, payload, db, rd).await,
        "waiting" => on_waiting(context, payload, db, rd).await,
        _ => unreachable!()
    }
}

///
///
///
async fn on_active(context: String, _payload: JValue, db: &DbPool, rd: &Client) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);
    let mut rd_conn = rd.get_async_connection().await.expect(MSG_NO_RD_CONNECTION);

    debug!("Handling 'active' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;
    let instructions_json: String = rd_conn.get(format!("{}_instructions", context)).await?;
    let session_uuid: String = rd_conn.get(format!("{}_session", context)).await?;

    // Setup VM
    let cursor = RedisCursor::new(format!("{}_cursor", context), rd);
    let environment = RedisEnvironment::new(format!("{}_env", context), None, rd);
    let instructions = serde_json::from_str(&instructions_json)?;
    let system = LocalSystem::new(session_uuid.parse()?);
    let vault = InMemoryVault::new(Default::default());
    let mut machine = AsyncMachine::new(
        instructions, 
        invocation_id,
        Box::new(cursor), 
        Box::new(environment),
        Box::new(system),
        Box::new(vault)
    );
    
    // Run
    let result = machine.walk().await?;
    let status = match result {
        MachineResult::Complete => "complete",
        MachineResult::Waiting => "waiting",
    };

    // Update invocation status
    diesel::update(inv_db::invocations.find(invocation_id))
        .set(inv_db::status.eq(status))
        .get_result::<Invocation>(&db_conn)?;

    Ok(vec!((context, String::from(format!(r#"{{"event": "{}"}}"#, status)))))
}

///
///
///
async fn on_callback(context: String, payload: JValue, db: &DbPool, rd: &Client) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);
    let mut rd_conn = rd.get_async_connection().await.expect(MSG_NO_RD_CONNECTION);

    debug!("Handling 'callback' event within context: {}", context);
    debug!("Callback payload: {:#?}", payload);

    let invocation_id: i32 = context[4..].parse()?;
    let instructions_json: String = rd_conn.get(format!("{}_instructions", context)).await?;
    let session_uuid: String = rd_conn.get(format!("{}_session", context)).await?;

    // Setup VM
    let cursor = RedisCursor::new(format!("{}_cursor", context), rd);
    let environment = RedisEnvironment::new(format!("{}_env", context), None, rd);
    let instructions = serde_json::from_str(&instructions_json)?;
    let system = LocalSystem::new(session_uuid.parse()?);
    let vault = InMemoryVault::new(Default::default());
    let mut machine = AsyncMachine::new(
        instructions, 
        invocation_id,
        Box::new(cursor), 
        Box::new(environment),
        Box::new(system),
        Box::new(vault)
    );

    // Run
    machine.callback(Value::Unit)?;

    // Update invocation status
    diesel::update(inv_db::invocations.find(invocation_id))
        .set(inv_db::status.eq("active"))
        .get_result::<Invocation>(&db_conn)?;

    Ok(vec!((context, String::from(format!(r#"{{"event": "active"}}"#)))))
}

///
///
///
async fn on_complete(context: String, _payload: JValue, db: &DbPool, rd: &Client) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);

    debug!("Handling 'complete' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;
    let invocation = inv_db::invocations.find(invocation_id).first::<Invocation>(&db_conn)?;

    // Store (or update) non-temporary variables in DB
    let environment = RedisEnvironment::new(format!("{}_env", context), None, rd);
    let variables = environment.variables();
    debug!("Variables: {:?}", variables);

    let variables = variables.iter().filter(|(k, _)| !k.starts_with("_"));
    // Only variables that are new or updated: after each set, append key to Redis set. 

    for (key, value) in variables {
        let value_json = serde_json::to_string(value)?;
        let new_variable = NewVariable::new(
            invocation.session, 
            key.clone(), 
            value.data_type().to_string(), 
            Some(value_json.clone())
        );

        diesel::insert_into(var_db::variables)
            .values(&new_variable)
            .on_conflict((var_db::name, var_db::session))
            .do_update()
            .set((var_db::content_json.eq(value_json), var_db::updated.eq(Utc::now().naive_utc())))
            .execute(&db_conn)?;
    }
    
    Ok(vec!())
}

///
///
///
async fn on_created(context: String, _payload: JValue, db: &DbPool, rd: &Client) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);
    let mut rd_conn = rd.get_async_connection().await.expect(MSG_NO_RD_CONNECTION);

    debug!("Handling 'created' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;
    let invocation = inv_db::invocations.find(invocation_id).first::<Invocation>(&db_conn)?;
    let session = ses_db::sessions.find(invocation.session).first::<Session>(&db_conn)?;
    let variables = var_db::variables.filter(var_db::session.eq(invocation.session)).load::<Variable>(&db_conn)?;

    // Store control variables in Redis
    let _ = rd_conn.set(format!("{}_session", context), &session.uuid).await?;
    let _ = rd_conn.set(format!("{}_instructions", context), &invocation.instructions_json).await?;

    // Store session variables in Redis
    let mut environment = RedisEnvironment::new(format!("{}_env", context), None, rd);
    for variable in variables {
        if let Some(content_json) = variable.content_json {
            environment.set(&variable.name, &serde_json::from_str::<Value>(&content_json)?);
        }
    }

    // Update invocation status
    diesel::update(inv_db::invocations.find(invocation_id))
        .set(inv_db::status.eq("ready"))
        .get_result::<Invocation>(&db_conn)?;

    Ok(vec!((context, String::from(r#"{"event": "ready"}"#))))
}

///
///
///
async fn on_ready(context: String, _payload: JValue, db: &DbPool, _rd: &Client) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);

    debug!("Handling 'ready' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;

    // Update invocation status
    diesel::update(inv_db::invocations.find(invocation_id))
        .set(inv_db::status.eq("active"))
        .get_result::<Invocation>(&db_conn)?;

    Ok(vec!((context, String::from(r#"{"event": "active"}"#))))
}

///
///
///
async fn on_waiting(context: String, _payload: JValue, _db: &DbPool, _rd: &Client) -> Result<Vec<Event>> {
    debug!("Handling 'waiting' event within context: {}", context);

    Ok(vec!())
}
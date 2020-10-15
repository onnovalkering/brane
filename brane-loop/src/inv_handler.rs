use crate::models::{Invocation, NewVariable, Session, Variable};
use crate::schema::{invocations::dsl as inv_db, sessions::dsl as ses_db, variables::dsl as var_db};
use anyhow::Result;
use brane_sys::{kubernetes::K8sSystem, local::LocalSystem, System};
use brane_vm::cursor::RedisCursor;
use brane_vm::environment::{Environment, RedisEnvironment};
use brane_vm::machine::{AsyncMachine, MachineResult};
use brane_vm::vault::{HashiVault, InMemoryVault, Vault};
use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use redis::AsyncCommands;
use redis::Client;
use serde_json::json;
use serde_json::Value as JValue;
use specifications::common::Value;
use specifications::instructions::Instruction;
use std::env;

type DbPool = Pool<ConnectionManager<PgConnection>>;
type Event = (String, String);

lazy_static! {
    static ref SYSTEM: String = env::var("SYSTEM").unwrap_or_else(|_| String::from("local"));
}

const MSG_NO_DB_CONNECTION: &str = "Couldn't get connection from db pool.";
const MSG_NO_RD_CONNECTION: &str = "Couldn't get connection from rd client.";

///
///
///
pub async fn handle(
    context: String,
    payload: JValue,
    db: &DbPool,
    rd: &Client,
) -> Result<Vec<Event>> {
    let event = payload["event"]
        .as_str()
        .expect("Payload doesn't contain 'event' property.");

    match event {
        "active" => on_active(context, payload, db, rd).await,
        "callback" => on_callback(context, payload, db, rd).await,
        "complete" => on_complete(context, payload, db, rd).await,
        "created" => on_created(context, payload, db, rd).await,
        "ready" => on_ready(context, payload, db, rd).await,
        "waiting" => on_waiting(context, payload, db, rd).await,
        _ => unreachable!(),
    }
}

///
///
///
async fn on_active(
    context: String,
    _payload: JValue,
    db: &DbPool,
    rd: &Client,
) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);
    let mut rd_conn = rd.get_async_connection().await.expect(MSG_NO_RD_CONNECTION);

    debug!("Handling 'active' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;
    let instructions_json: String = rd_conn.get(format!("{}_instructions", context)).await?;
    let session_uuid: String = rd_conn.get(format!("{}_session", context)).await?;

    // Setup VM
    let instructions = serde_json::from_str(&instructions_json)?;
    let mut machine = setup_machine(&context, session_uuid, invocation_id, instructions, rd)?;

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

    Ok(vec![(context, String::from(format!(r#"{{"event": "{}"}}"#, status)))])
}

///
///
///
async fn on_callback(
    context: String,
    payload: JValue,
    db: &DbPool,
    rd: &Client,
) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);
    let mut rd_conn = rd.get_async_connection().await.expect(MSG_NO_RD_CONNECTION);

    debug!("Handling 'callback' event within context: {}", context);
    debug!("Callback payload: {}", payload);

    let invocation_id: i32 = context[4..].parse()?;
    let instructions_json: String = rd_conn.get(format!("{}_instructions", context)).await?;
    let session_uuid: String = rd_conn.get(format!("{}_session", context)).await?;

    // TODO: verify that status of invocation is 'waiting', otherwise discard (illegal or outdated).

    // Setup VM
    let instructions = serde_json::from_str(&instructions_json)?;
    let mut machine = setup_machine(&context, session_uuid, invocation_id, instructions, rd)?;

    // Run
    let value = serde_json::from_value(payload["value"].clone())?;
    machine.callback(value)?;

    // Update invocation status
    diesel::update(inv_db::invocations.find(invocation_id))
        .set(inv_db::status.eq("active"))
        .get_result::<Invocation>(&db_conn)?;

    Ok(vec![(context, String::from(format!(r#"{{"event": "active"}}"#)))])
}

///
///
///
async fn on_complete(
    context: String,
    _payload: JValue,
    db: &DbPool,
    rd: &Client,
) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);

    debug!("Handling 'complete' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;
    let invocation = inv_db::invocations.find(invocation_id).first::<Invocation>(&db_conn)?;

    // Store (or update) non-temporary variables in DB
    let environment = RedisEnvironment::new(format!("{}_env", context), None, rd);
    let variables = environment.variables();
    debug!("Variables: {:?}", variables);

    // Store 'terminate' variable as invocation output
    if let Some(terminate) = variables.get("terminate") {
        let return_json = serde_json::to_string(terminate)?;
        diesel::update(inv_db::invocations.find(invocation_id))
            .set(inv_db::return_json.eq(return_json))
            .get_result::<Invocation>(&db_conn)?;
    }

    let terminate = variables.get("terminate").unwrap_or(&Value::Unit).clone();
    let variables = variables.iter().filter(|(k, _)| !k.starts_with("_"));

    // Only variables that are new or updated: after each set, append key to Redis set.
    for (key, value) in variables {
        if key == "terminate" {
            continue;
        }

        let value_json = serde_json::to_string(value)?;
        let new_variable = NewVariable::new(
            invocation.session,
            key.clone(),
            value.data_type().to_string(),
            Some(value_json.clone()),
        )
        .unwrap();

        diesel::insert_into(var_db::variables)
            .values(&new_variable)
            .on_conflict((var_db::name, var_db::session))
            .do_update()
            .set((
                var_db::content_json.eq(value_json),
                var_db::type_.eq(value.data_type()),
                var_db::updated.eq(Utc::now().naive_utc()),
            ))
            .execute(&db_conn)?;
    }

    // Update invocation status
    let stopped = Utc::now().naive_utc();
    diesel::update(inv_db::invocations.find(invocation_id))
        .set(inv_db::stopped.eq(stopped))
        .get_result::<Invocation>(&db_conn)?;

    let session = ses_db::sessions.find(invocation.session).first::<Session>(&db_conn)?;
    debug!("session: {:?}", session.parent);

    if let Some(parent) = session.parent {
        let context = format!("inv-{}", parent);
        let payload = json!({
            "event": "callback",
            "value": terminate,
        })
        .to_string();

        Ok(vec![(context, payload)])
    } else {
        Ok(vec![])
    }
}

///
///
///
async fn on_created(
    context: String,
    _payload: JValue,
    db: &DbPool,
    rd: &Client,
) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);
    let mut rd_conn = rd.get_async_connection().await.expect(MSG_NO_RD_CONNECTION);

    debug!("Handling 'created' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;
    let invocation = inv_db::invocations.find(invocation_id).first::<Invocation>(&db_conn)?;
    let session = ses_db::sessions.find(invocation.session).first::<Session>(&db_conn)?;
    let variables = var_db::variables
        .filter(var_db::session.eq(invocation.session))
        .load::<Variable>(&db_conn)?;

    // Store control variables in Redis
    let _ = rd_conn.set(format!("{}_session", context), &session.uuid).await?;
    let _ = rd_conn
        .set(format!("{}_instructions", context), &invocation.instructions_json)
        .await?;

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

    Ok(vec![(context, String::from(r#"{"event": "ready"}"#))])
}

///
///
///
async fn on_ready(
    context: String,
    _payload: JValue,
    db: &DbPool,
    _rd: &Client,
) -> Result<Vec<Event>> {
    let db_conn = db.get().expect(MSG_NO_DB_CONNECTION);

    debug!("Handling 'ready' event within context: {}", context);

    let invocation_id: i32 = context[4..].parse()?;

    // Update invocation status
    let started = Utc::now().naive_utc();
    diesel::update(inv_db::invocations.find(invocation_id))
        .set((inv_db::status.eq("active"), inv_db::started.eq(started)))
        .get_result::<Invocation>(&db_conn)?;

    Ok(vec![(context, String::from(r#"{"event": "active"}"#))])
}

///
///
///
async fn on_waiting(
    context: String,
    _payload: JValue,
    _db: &DbPool,
    _rd: &Client,
) -> Result<Vec<Event>> {
    debug!("Handling 'waiting' event within context: {}", context);

    Ok(vec![])
}

///
///
///
fn setup_machine(
    context: &String,
    session_uuid: String,
    invocation_id: i32,
    instructions: Vec<Instruction>,
    rd: &Client,
) -> Result<AsyncMachine> {
    let cursor = RedisCursor::new(format!("{}_cursor", context), rd);
    let environment = RedisEnvironment::new(format!("{}_env", context), None, rd);
    let system = setup_system(session_uuid)?;
    let vault = setup_vault()?;

    let machine = AsyncMachine::new(
        instructions,
        invocation_id,
        Box::new(cursor),
        Box::new(environment),
        system,
        vault,
    );

    Ok(machine)
}

///
///
///
fn setup_system(session_uuid: String) -> Result<Box<dyn System>> {
    let system: Box<dyn System> = match SYSTEM.as_str() {
        "local" => Box::new(LocalSystem::new(session_uuid.parse()?)),
        "kubernetes" => Box::new(K8sSystem::new(session_uuid.parse()?)),
        "xenon" => unimplemented!(),
        _ => bail!("Unrecognized system: {}", SYSTEM.as_str()),
    };

    Ok(system)
}

///
///
///
fn setup_vault() -> Result<Box<dyn Vault>> {
    if let Ok(vault_host) = env::var("VAULT_HOST") {
        let vault_url = format!("http://{}", vault_host);
        let vault_token = env::var("VAULT_TOKEN")?;

        Ok(Box::new(HashiVault::new(vault_url, vault_token)))
    } else {
        Ok(Box::new(InMemoryVault::new(Default::default())))
    }
}

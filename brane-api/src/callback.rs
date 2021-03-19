use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse, FromRequest};
use anyhow::Result;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::Deserialize;
use serde_json::json;
use specifications::{common::Value, status::StatusInfo};

const TOPIC_CONTROL: &str = "control";

///
///
///
pub fn scope() -> Scope {
    web::scope("/callback")
        .app_data(web::Json::<ActCallback>::configure(|cfg| {
            cfg.limit(1024000)
        }))
        .route("act", web::post().to(act_callback))
        .route("status", web::post().to(status_callback))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActCallback {
    pub invocation_id: i32,
    pub value: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusCallback {
    pub invocation_id: i32,
    pub info: StatusInfo,
}

///
///
///
async fn act_callback(
    _req: HttpRequest,
    producer: web::Data<FutureProducer>,
    json: web::Json<ActCallback>,
) -> HttpResponse {
    let message_key = format!("inv-{}", json.invocation_id);
    let message_payload = json!({
        "event": "callback",
        "value": json.value,
    })
    .to_string();

    let delivery = trigger_event(&producer, message_key, message_payload).await;
    if delivery.is_ok() {
        HttpResponse::Accepted().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn status_callback(
    _req: HttpRequest,
    producer: web::Data<FutureProducer>,
    json: web::Json<StatusCallback>,
) -> HttpResponse {
    let message_key = format!("inv-{}", json.invocation_id);
    let message_payload = json!({
        "event": "status",
        "info": json.info,
    })
    .to_string();

    let delivery = trigger_event(&producer, message_key, message_payload).await;
    if delivery.is_ok() {
        HttpResponse::Accepted().body("")
    } else {
        HttpResponse::InternalServerError().body("")
    }
}

///
///
///
async fn trigger_event(
    producer: &FutureProducer,
    context: String,
    payload: String,
) -> Result<()> {
    let message = FutureRecord::to(TOPIC_CONTROL).key(&context).payload(&payload);

    let delivery = producer.send(message, Timeout::Never).await;
    match delivery {
        Ok(_) => Ok(()),
        Err(error) => Err(anyhow!(
            "Unable to tirgger event within context '{}':\n{:?}",
            context,
            error,
        )),
    }
}

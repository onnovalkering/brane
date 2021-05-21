use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;

lazy_static! {
    static ref VAULT_URL: String = format!(
        "http://{}",
        env::var("VAULT_HOST").unwrap_or_else(|_| String::from("vault:8020"))
    );
    static ref VAULT_TOKEN: String = env::var("VAULT_TOKEN").unwrap_or_else(|_| String::from("mytoken"));
}

///
///
///
pub fn scope() -> Scope {
    web::scope("/vault")
        .route("", web::get().to(get_secrets))
        .route("", web::post().to(create_secret))
        .route("{name}", web::get().to(get_secret))
        .route("{name}", web::put().to(update_secret))
        .route("{name}", web::delete().to(delete_secret))
}

#[derive(Serialize, Deserialize)]
struct Secrets {
    data: Option<SecretsData>,
}

#[derive(Serialize, Deserialize, Default)]
struct SecretsData {
    keys: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct Secret {
    data: SecretData,
}

#[derive(Serialize, Deserialize)]
struct SecretData {
    data: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CreateSecret {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct UpdateSecret {
    pub value: String,
}

///
///
///
async fn get_secrets(_req: HttpRequest) -> HttpResponse {
    let client = reqwest::blocking::Client::new();
    let secrets: Secrets = client
        .get(&format!("{}/v1/secret/metadata?list=true", VAULT_URL.as_str()))
        .header("X-Vault-Token", VAULT_TOKEN.as_str())
        .send()
        .unwrap()
        .json()
        .unwrap();

    let response = secrets.data.unwrap_or_default();
    HttpResponse::Ok().json(response)
}

///
///
///
async fn get_secret(
    _req: HttpRequest,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let client = reqwest::blocking::Client::new();
    let secret: Secret = client
        .get(&format!("{}/v1/secret/data/{}", VAULT_URL.as_str(), path.0))
        .header("X-Vault-Token", VAULT_TOKEN.as_str())
        .send()
        .unwrap()
        .json()
        .unwrap();

    HttpResponse::Ok().json(secret.data)
}

///
///
///
async fn create_secret(
    _req: HttpRequest,
    json: web::Json<CreateSecret>,
) -> HttpResponse {
    let client = reqwest::blocking::Client::new();
    let _ = client
        .post(&format!("{}/v1/secret/data/{}", VAULT_URL.as_str(), json.key))
        .header("X-Vault-Token", VAULT_TOKEN.as_str())
        .json(&json!({
            "data": {
                "value": json.value
            }
        }))
        .send()
        .unwrap();

    HttpResponse::Ok().body("")
}

///
///
///
async fn update_secret(
    _req: HttpRequest,
    path: web::Path<(String,)>,
    json: web::Json<UpdateSecret>,
) -> HttpResponse {
    let payload = json!({
        "data": {
            "value": json.value
        }
    });

    let client = reqwest::blocking::Client::new();
    let _ = client
        .put(&format!("{}/v1/secret/data/{}", VAULT_URL.as_str(), path.0))
        .header("X-Vault-Token", VAULT_TOKEN.as_str())
        .json(&payload)
        .send()
        .unwrap();

    HttpResponse::Ok().body("")
}

///
///
///
async fn delete_secret(
    _req: HttpRequest,
    path: web::Path<(String,)>,
) -> HttpResponse {
    let client = reqwest::blocking::Client::new();
    let res = client
        .delete(&format!("{}/v1/secret/metadata/{}", VAULT_URL.as_str(), path.0))
        .header("X-Vault-Token", VAULT_TOKEN.as_str())
        .send()
        .unwrap();

    println!("{:?}", res);

    HttpResponse::Ok().body("")
}

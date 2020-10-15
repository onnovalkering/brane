use actix_web::Scope;
use actix_web::{web, HttpRequest, HttpResponse};

///
///
///
pub fn scope() -> Scope {
    web::scope("/health").route("", web::get().to(get_health))
}

///
///
///
async fn get_health(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("OK!\n")
}

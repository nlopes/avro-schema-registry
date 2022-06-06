use actix_web::{HttpResponse, Responder};

pub async fn status() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"status\": \"healthy\"}")
}

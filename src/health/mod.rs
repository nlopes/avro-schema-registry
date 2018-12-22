use actix_web::{HttpRequest, HttpResponse};

pub fn metrics(_req: HttpRequest) -> HttpResponse {
    unimplemented!();
}

pub fn status(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"status\": \"healthy\"}")
}

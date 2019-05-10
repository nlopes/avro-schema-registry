use actix_web::HttpResponse;

pub fn status() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"status\": \"healthy\"}")
}

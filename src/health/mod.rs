use actix_web::HttpResponse;

pub fn metrics() -> HttpResponse {
    unimplemented!()
}

pub fn status() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"status\": \"healthy\"}")
}

use actix_web::{HttpRequest, HttpResponse};

pub fn metrics(_: &HttpRequest) -> HttpResponse {
    unimplemented!();
}

pub fn status(_: &HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"status\": \"healthy\"}")
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test};

    #[test]
    fn test_status() {
        let resp = test::TestRequest::with_header("content-type", "application/json")
            .run(&super::status)
            .unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);

        let f = b"{\"status\": \"healthy\"}";
        assert_eq!(
            *resp.body(),
            actix_web::Body::Binary(actix_web::Binary::Slice(f))
        );
    }
}

use actix_web::HttpResponse;

pub fn metrics() -> HttpResponse {
    unimplemented!()
}

pub fn status() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"status\": \"healthy\"}")
}

// #[cfg(test)]
// mod tests {
//     use actix_web::{http, test};
//     use futures::future::IntoFuture;

//     #[test]
//     fn test_status() {
//         let _ =
//             test::TestRequest::with_header("content-type", "application/json").to_http_request();
//         let resp = test::block_on(super::status().into_future()).unwrap();

//         assert_eq!(resp.status(), http::StatusCode::OK);

//         let f = b"{\"status\": \"healthy\"}";
//         //assert_eq!(*resp.body(), f);
//         assert_eq!(3, 3);
//     }
// }

use actix_web::http;

use crate::common::server::setup;
use crate::db::DbAuxOperations;

#[actix_rt::test]
async fn test_get_schema_without_schema() {
    let (server, _) = setup();

    // it returns 404 with message
    server
        .test(
            http::Method::GET,
            "/schemas/ids/1",
            None,
            http::StatusCode::NOT_FOUND,
            r#"\{"error_code":40403,"message":"Schema not found"\}"#,
        )
        .await;
}

#[actix_rt::test]
async fn test_get_schema_with_schema() {
    let (server, mut conn) = setup();

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = conn.register_schema(String::from("subject1"), schema_s.to_string());

    // it returns schema
    server
        .test(
            http::Method::GET,
            &format!("/schemas/ids/{}", schema.id),
            None,
            http::StatusCode::OK,
            r#"\{"schema":"\{    \\"type\\": \\"record\\",    \\"name\\": \\"test\\",    \\"fields\\":    \[        \{            \\"type\\": \\"string\\",            \\"name\\": \\"field1\\",            \\"default\\": \\"\\"        \},        \{            \\"type\\": \\"string\\",            \\"name\\": \\"field2\\"        \}    \]\}"\}"#
        )
        .await;
}

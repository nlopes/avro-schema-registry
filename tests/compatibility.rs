use actix_web::{http, rt as actix_rt};

use crate::common::server::setup;
use crate::db::DbAuxOperations;
use avro_schema_registry::api::SchemaBody;

#[actix_rt::test]
async fn test_schema_for_compatibility_with_non_existent_subject() {
    let (server, _) = setup();
    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema)),
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40401,\"message\":\"Subject not found\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_schema_for_compatibility_with_subject_and_with_non_existent_version() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("FORWARD");

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };

    // it returns 404 with 'Version not found'
    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/2",
            Some(json!(schema)),
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40402,\"message\":\"Version not found\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_schema_for_compatibility_with_subject_and_with_version() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("FORWARD");

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema_s2 = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();

    let _ = conn.register_schema(String::from("test.subject"), schema_s.to_string());
    let schema2 = SchemaBody { schema: schema_s2 };

    // it returns 200 with is_compatible: true on Backward compatibility
    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema2)),
            http::StatusCode::OK,
            "{\"is_compatible\":false}",
        )
        .await;
}

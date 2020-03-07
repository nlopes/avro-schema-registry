use actix_web::http;

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
            http::StatusCode::NOT_IMPLEMENTED,
            "",
        )
        .await;
}

#[actix_rt::test]
async fn test_schema_for_compatibility_with_non_existent_version() {
    let (server, conn) = setup();

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };

    conn.create_test_subject_with_config("FULL");

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/2",
            Some(json!(schema)),
            http::StatusCode::NOT_IMPLEMENTED,
            "",
        )
        .await;
}

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
async fn test_schema_for_forward_compatibility_with_subject_and_with_version() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("FORWARD");

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema_forward_compatible_s =
        std::fs::read_to_string("tests/fixtures/schema_forward_compatible.json").unwrap();

    let _ = conn.register_schema(String::from("test.subject"), schema_s.to_string());
    let schema_forward_compatible = SchemaBody {
        schema: schema_forward_compatible_s.to_string(),
    };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema_forward_compatible)),
            http::StatusCode::OK,
            "{\"is_compatible\":true}",
        )
        .await;

    // Should not be backwards compatible
    let schema_backward_compatible_s =
        std::fs::read_to_string("tests/fixtures/schema_backward_compatible.json").unwrap();

    let schema_backward_compatible = SchemaBody {
        schema: schema_backward_compatible_s.to_string(),
    };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema_backward_compatible)),
            http::StatusCode::OK,
            "{\"is_compatible\":false}",
        )
        .await;
}

#[actix_rt::test]
async fn test_schema_for_backward_compatibility_with_subject_and_with_version() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("BACKWARD");

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema_backward_compatible_s =
        std::fs::read_to_string("tests/fixtures/schema_backward_compatible.json").unwrap();

    let _ = conn.register_schema(String::from("test.subject"), schema_s.to_string());
    let schema2 = SchemaBody {
        schema: schema_backward_compatible_s.to_string(),
    };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema2)),
            http::StatusCode::OK,
            "{\"is_compatible\":true}",
        )
        .await;

    let schema_forward_compatible_s =
        std::fs::read_to_string("tests/fixtures/schema_forward_compatible.json").unwrap();

    let schema_forward_compatible = SchemaBody {
        schema: schema_forward_compatible_s.to_string(),
    };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema_forward_compatible)),
            http::StatusCode::OK,
            "{\"is_compatible\":false}",
        )
        .await;
}

#[actix_rt::test]
async fn test_schema_for_full_compatibility_with_subject_and_with_version() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("FULL");

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema_backward_compatible_s =
        std::fs::read_to_string("tests/fixtures/schema_backward_compatible.json").unwrap();

    let _ = conn.register_schema(String::from("test.subject"), schema_s.to_string());
    let schema2 = SchemaBody {
        schema: schema_backward_compatible_s.to_string(),
    };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema2)),
            http::StatusCode::OK,
            "{\"is_compatible\":false}",
        )
        .await;

    let schema_forward_compatible_s =
        std::fs::read_to_string("tests/fixtures/schema_forward_compatible.json").unwrap();

    let schema_forward_compatible = SchemaBody {
        schema: schema_forward_compatible_s.to_string(),
    };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema_forward_compatible)),
            http::StatusCode::OK,
            "{\"is_compatible\":false}",
        )
        .await;

    let schema_full_compatible_s =
        std::fs::read_to_string("tests/fixtures/schema_full_compatible.json").unwrap();

    let schema_full_compatible = SchemaBody {
        schema: schema_full_compatible_s.to_string(),
    };

    server
        .test(
            http::Method::POST,
            "/compatibility/subjects/test.subject/versions/1",
            Some(json!(schema_full_compatible)),
            http::StatusCode::OK,
            "{\"is_compatible\":true}",
        )
        .await;
}

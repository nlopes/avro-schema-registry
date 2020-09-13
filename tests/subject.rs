use actix_web::{http, rt as actix_rt};

use crate::common::server::setup;
use crate::db::DbAuxOperations;
use avro_schema_registry::api::SchemaBody;

#[actix_rt::test]
async fn test_get_subjects_without_subjects() {
    let (server, conn) = setup();
    conn.reset_subjects();
    // returns empty list
    server
        .test(
            http::Method::GET,
            "/subjects",
            None,
            http::StatusCode::OK,
            "[]",
        )
        .await;
}

#[actix_rt::test]
async fn test_get_subjects_with_subjects() {
    let (server, conn) = setup();
    conn.reset_subjects();
    conn.add_subjects(vec![String::from("subject1"), String::from("subject2")]);

    // it returns list of subjects
    server
        .test(
            http::Method::GET,
            "/subjects",
            None,
            http::StatusCode::OK,
            "[\"subject1\",\"subject2\"]",
        )
        .await;
}

#[actix_rt::test]
async fn test_get_versions_under_subject_without_subject() {
    let (server, _) = setup();
    // it returns 404 with 'Subject not found'
    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions",
            None,
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40401,\"message\":\"Subject not found\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_get_versions_under_subject_without_versions() {
    let (server, conn) = setup();
    conn.add_subjects(vec![String::from("test.subject")]);

    // This should never happen with correct usage of the API
    // it returns 404 with 'Subject not found'
    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions",
            None,
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40401,\"message\":\"Subject not found\"}",
        )
        .await;

    conn.reset_subjects();
}

#[actix_rt::test]
async fn test_get_versions_under_subject_with_versions() {
    let (server, _) = setup();
    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;

    // it returns list of one
    // TODO(nlopes): dangerous, postgresql can pick any other ID
    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions",
            None,
            http::StatusCode::OK,
            "[1]",
        )
        .await;

    // it returns list of many
    let schema2_s = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();
    let schema2 = SchemaBody { schema: schema2_s };

    // This modifies the database state in preparation for the next request
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema2)),
            http::StatusCode::OK,
            "",
        )
        .await;

    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions",
            None,
            http::StatusCode::OK,
            "[1,2]",
        )
        .await;
}

#[actix_rt::test]
async fn test_delete_subject_without_subject() {
    let (server, _) = setup();
    // it returns 404 with 'Subject not found'
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject",
            None,
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40401,\"message\":\"Subject not found\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_delete_subject_with_subject() {
    let (server, _) = setup();
    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;

    // it returns list with versions of schemas deleted
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject",
            None,
            http::StatusCode::OK,
            "[1]",
        )
        .await;
}

#[actix_rt::test]
async fn test_get_version_of_schema_registered_under_subject_without_subject() {
    let (server, _) = setup();
    // it returns 404 with 'Subject not found'
    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions/1",
            None,
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40401,\"message\":\"Subject not found\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_get_version_of_schema_registered_under_subject_with_subject() {
    let (server, _) = setup();
    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;

    // with non existing version it returns 404 with 'Version not found'
    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions/2",
            None,
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40402,\"message\":\"Version not found\"}",
        )
        .await;
    // with version out of bounds it returns 422 with 'Invalid version' for lower bound
    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions/0",
            None,
            http::StatusCode::UNPROCESSABLE_ENTITY,
            "{\"error_code\":42202,\"message\":\"Invalid version\"}",
        )
        .await;

    // with version out of bounds it returns 422 with 'Invalid version' for upper bound
    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions/2147483648",
            None,
            http::StatusCode::UNPROCESSABLE_ENTITY,
            "{\"error_code\":42202,\"message\":\"Invalid version\"}",
        )
        .await;

    // with latest version it returns version with schema
    // TODO(nlopes): fix expect body - requires knowing that the id
    // below is 86 - maybe direct sql query
    //
    //"{\"subject\":\"test.subject\",\"id\":86,\"version\":1,\"schema\":\"{\\n    \\\"type\\\": \\\"record\\\",\\n    \\\"name\\\": \\\"test\\\",\\n    \\\"fields\\\":\\n    [\\n        {\\n            \\\"type\\\": \\\"string\\\",\\n             \\\"name\\\": \\\"field1\\\"\\n           },\\n           {\\n             \\\"type\\\": \\\"int\\\",\\n             \\\"name\\\": \\\"field2\\\"\\n           }\\n         ]\\n}\\n\"}";

    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions/latest",
            None,
            http::StatusCode::OK,
            "",
        )
        .await;

    // with existing version it returns version with schema
    // TODO(nlopes): fix expect body - requires knowing that the id
    // below is 86 - maybe direct sql query
    //
    //"{\"subject\":\"test.subject\",\"id\":86,\"version\":1,\"schema\":\"{\\n    \\\"type\\\": \\\"record\\\",\\n    \\\"name\\\": \\\"test\\\",\\n    \\\"fields\\\":\\n    [\\n        {\\n            \\\"type\\\": \\\"string\\\",\\n             \\\"name\\\": \\\"field1\\\"\\n           },\\n           {\\n             \\\"type\\\": \\\"int\\\",\\n             \\\"name\\\": \\\"field2\\\"\\n           }\\n         ]\\n}\\n\"}";

    server
        .test(
            http::Method::GET,
            "/subjects/test.subject/versions/1",
            None,
            http::StatusCode::OK,
            "",
        )
        .await;
}

#[actix_rt::test]
async fn test_register_schema_under_subject_with_valid_schema() {
    let (server, _) = setup();

    // it returns schema identifier
    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };

    // TODO(nlopes): Check for body "{\"id\":\"147\"}"
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;
}

#[actix_rt::test]
async fn test_register_schema_under_subject_with_invalid_schema() {
    let (server, _) = setup();
    let schema = SchemaBody {
        schema: "{}".to_string(),
    };

    // it returns 422 with 'Invalid Avro schema'
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::UNPROCESSABLE_ENTITY,
            "{\"error_code\":42201,\"message\":\"Invalid Avro schema\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_check_schema_registration_without_subject() {
    let (server, _) = setup();
    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };

    // it returns 404 with 'Subject not found'
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject",
            Some(json!(schema)),
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40401,\"message\":\"Subject not found\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_check_schema_registration_with_subject_but_different_schema() {
    let (server, _) = setup();

    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema2_s = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();
    let schema = SchemaBody { schema: schema_s };
    let schema2 = SchemaBody { schema: schema2_s };

    // setup of schema 2
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema2)),
            http::StatusCode::OK,
            "",
        )
        .await;

    // it returns 404 with Subject not found
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject",
            Some(json!(schema)),
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40403,\"message\":\"Schema not found\"}",
        )
        .await;
}

#[actix_rt::test]
async fn test_delete_schema_version_under_subject_without_subject() {
    let (server, _) = setup();
    // it returns 404 with 'Subject not found'
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject/versions/1",
            None,
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40401,\"message\":\"Subject not found\"}",
        )
        .await;
}
#[actix_rt::test]
async fn test_delete_schema_version_under_subject_with_subject() {
    let (server, _) = setup();
    // setup
    let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
    let schema = SchemaBody { schema: schema_s };
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;

    // with non existing version it returns 404 with 'Version not found'
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject/versions/2",
            None,
            http::StatusCode::NOT_FOUND,
            "{\"error_code\":40402,\"message\":\"Version not found\"}",
        )
        .await;

    // with version out of bounds it returns 422 with 'Invalid version'
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject/versions/0",
            None,
            http::StatusCode::UNPROCESSABLE_ENTITY,
            "{\"error_code\":42202,\"message\":\"Invalid version\"}",
        )
        .await;
    // with existing version it returns list with versions of schemas deleted
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject/versions/1",
            None,
            http::StatusCode::OK,
            "1",
        )
        .await;

    // re-setup before testing with latest
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;
    // with latest version and only one version it returns version of schema deleted
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject/versions/latest",
            None,
            http::StatusCode::OK,
            "1",
        )
        .await;
    // setup for next test
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;
    let schema_s = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();
    let schema = SchemaBody { schema: schema_s };
    server
        .test(
            http::Method::POST,
            "/subjects/test.subject/versions",
            Some(json!(schema)),
            http::StatusCode::OK,
            "",
        )
        .await;
    // with latest version and with multiple versions it returns version of schema deleted
    server
        .test(
            http::Method::DELETE,
            "/subjects/test.subject/versions/latest",
            None,
            http::StatusCode::OK,
            "2",
        )
        .await;
}

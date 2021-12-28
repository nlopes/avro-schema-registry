use actix_web::http;

use crate::common::server::setup;
use crate::db::DbAuxOperations;

#[actix_rt::test]
async fn test_get_global_config() {
    let (server, _) = setup();

    // returns compatibility
    server
        .test(
            http::Method::GET,
            "/config",
            None,
            http::StatusCode::OK,
            r#"\{"compatibility":"BACKWARD"\}"#,
        )
        .await;
}

#[actix_rt::test]
async fn test_set_global_config_with_valid_compatibility_full() {
    let (server, _) = setup();

    // returns compatibility
    server
        .test(
            http::Method::PUT,
            "/config",
            Some(json!({"compatibility": "FULL"})),
            http::StatusCode::OK,
            r#"\{"compatibility":"FULL"\}"#,
        )
        .await;
}

#[actix_rt::test]
async fn test_set_global_config_with_invalid_compatibility() {
    let (server, _) = setup();
    // returns 422 with Invalid compatibility level
    server
        .test(
            http::Method::PUT,
            "/config",
            Some(json!({"compatibility": "NOT_VALID"})),
            http::StatusCode::UNPROCESSABLE_ENTITY,
            r#"\{"error_code":42203,"message":"Invalid compatibility level"\}"#,
        )
        .await;
}

#[actix_rt::test]
async fn test_get_compatibility_level_with_existent_subject() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("FULL");

    // returns valid compatibility
    server
        .test(
            http::Method::GET,
            "/config/test.subject",
            None,
            http::StatusCode::OK,
            r#"\{"compatibility":"FULL"\}"#,
        )
        .await;
}

#[actix_rt::test]
async fn test_get_compatibility_level_with_non_existent_subject() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("FULL");
    conn.reset_subjects();

    // returns 404 with Invalid compatibility level
    server
        .test(
            http::Method::GET,
            "/config/test.subject",
            None,
            http::StatusCode::NOT_FOUND,
            r#"\{"error_code":40401,"message":"Subject not found"\}"#,
        )
        .await;
}

#[actix_rt::test]
async fn test_update_compatibility_level_with_existent_subject() {
    let (server, conn) = setup();
    conn.create_test_subject_with_config("FULL");

    // with valid compatibility FORWARD_TRANSITIVE it returns FORWARD_TRANSITIVE
    server
        .test(
            http::Method::PUT,
            "/config/test.subject",
            Some(json!({"compatibility": "FORWARD_TRANSITIVE"})),
            http::StatusCode::OK,
            r#"\{"compatibility":"FORWARD_TRANSITIVE"\}"#,
        )
        .await;

    // with invalid compatibility it returns 422
    server
        .test(
            http::Method::PUT,
            "/config/test.subject",
            Some(json!({"compatibility": "NOT_VALID"})),
            http::StatusCode::UNPROCESSABLE_ENTITY,
            r#"\{"error_code":42203,"message":"Invalid compatibility level"}"#,
        )
        .await;
}

#[actix_rt::test]
async fn test_update_compatibility_level_with_non_existent_subject() {
    let (server, conn) = setup();
    conn.reset_subjects();

    // with valid compatibility FULL it returns 404
    server
        .test(
            http::Method::PUT,
            "/config/test.subject",
            Some(json!({"compatibility": "FULL"})),
            http::StatusCode::NOT_FOUND,
            r#"\{"error_code":40401,"message":"Subject not found"\}"#,
        )
        .await;
}

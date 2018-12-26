use std::{env, panic};

use actix_web::{http, test, HttpMessage};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use futures::future::Future;

use crate::common::AvroHeaders;
use avro_schema_registry::app;

fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    reset_global_config();

    let result = panic::catch_unwind(|| test());
    assert!(result.is_ok())
}

fn reset_global_config() {
    use avro_schema_registry::db::models::schema::configs::dsl::*;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    conn.transaction::<_, diesel::result::Error, _>(|| {
        diesel::update(configs)
            .filter(id.eq(0))
            .set(compatibility.eq("BACKWARD"))
            .execute(&conn)
    })
    .unwrap();
}

#[test]
fn get_global_config() {
    run_test(|| {
        let mut srv = test::TestServer::with_factory(app::create_avro_api_app);
        let request = srv
            .client(http::Method::GET, "/config")
            .with_avro_headers()
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        let expected = b"{\"compatibility\":\"BACKWARD\"}";
        assert!(response.status().is_success());
        let body = response.body().wait().unwrap();
        assert_eq!(&body[..], expected);
    });
}

#[test]
fn set_global_config_with_valid_compatibility() {
    run_test(|| {
        let mut srv = test::TestServer::with_factory(app::create_avro_api_app);
        let request = srv
            .client(http::Method::PUT, "/config")
            .with_avro_headers()
            .json(json!({"compatibility": "FULL"}))
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        let expected = b"{\"compatibility\":\"FULL\"}";
        assert!(response.status().is_success());
        let body = response.body().wait().unwrap();
        assert_eq!(&body[..], expected);
    });
}

#[test]
fn set_global_config_with_invalid_compatibility() {
    run_test(|| {
        let mut srv = test::TestServer::with_factory(app::create_avro_api_app);
        let request = srv
            .client(http::Method::PUT, "/config")
            .with_avro_headers()
            .json(json!({"compatibility": "NOT_VALID"}))
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        let expected = "{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}";
        assert!(response.status().is_client_error());
        assert_eq!(response.status().as_u16(), 422);
        let body = response.body().wait().unwrap();
        let body_str = std::str::from_utf8(&body[..]).unwrap();
        assert_eq!(body_str, expected);
    });
}

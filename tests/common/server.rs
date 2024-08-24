use actix_test as test;
use actix_web::{
    error::PayloadError,
    http,
    web::{Bytes, Data},
    App,
};
use awc::{ClientRequest, ClientResponse};
use futures::{executor::block_on, stream::Stream};
use serde_json::Value as JsonValue;

use super::settings::get_schema_registry_password;
use crate::db::DbAuxOperations;
use avro_schema_registry::app;
use avro_schema_registry::db::{DbConnection, DbManage, DbPool};

pub struct ApiTesterServer(test::TestServer);

pub fn setup() -> (ApiTesterServer, DbConnection) {
    let server = ApiTesterServer::new();
    let mut conn = DbPool::new_pool(Some(1)).connection().unwrap();
    conn.reset();

    (server, conn)
}

impl ApiTesterServer {
    pub fn new() -> Self {
        Self(test::start(|| {
            App::new()
                .configure(app::monitoring_routing)
                .app_data(Data::new(DbPool::new_pool(Some(1))))
                .configure(app::api_routing)
        }))
    }

    pub async fn test(
        &self,
        method: http::Method,
        path: &str,
        request_body: Option<JsonValue>,
        expected_status: http::StatusCode,
        expected_body: &str,
    ) {
        let Self(server) = self;
        let req = server.request(method, server.url(path)).avro_headers();

        match request_body {
            Some(b) => req
                .send_json(&b)
                .await
                .unwrap()
                .validate(expected_status, expected_body),

            None => req
                .send()
                .await
                .unwrap()
                .validate(expected_status, expected_body),
        };
    }
}

trait AvroRequest {
    fn avro_headers(self) -> ClientRequest;
}

impl AvroRequest for ClientRequest {
    fn avro_headers(self) -> Self {
        self.insert_header((http::header::CONTENT_TYPE, "application/json"))
            .insert_header((http::header::ACCEPT, "application/vnd.schemaregistry+json"))
            .basic_auth("", get_schema_registry_password())
    }
}

trait ValidateResponse {
    fn validate(self, expected_status: http::StatusCode, expected_body: &str);
}

impl<S> ValidateResponse for ClientResponse<S>
where
    S: Stream<Item = Result<Bytes, PayloadError>> + Unpin,
{
    fn validate(mut self, expected_status: http::StatusCode, expected_body_regex: &str) {
        assert_eq!(self.status(), expected_status);
        let b = block_on(self.body()).unwrap();
        let s = b
            .iter()
            .map(|&c| c as char)
            .collect::<String>()
            .replace("\\r\\n", "")
            .replace("\\n", "");

        match regex::Regex::new(expected_body_regex) {
            Ok(re) => {
                assert!(
                    re.is_match(&s),
                    "{}",
                    format!("body doesn't match regex: {s} != {expected_body_regex}")
                )
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}

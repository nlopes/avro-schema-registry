use actix_http::error::PayloadError;
use actix_web::{
    client::{ClientRequest, ClientResponse},
    http, test,
    web::Bytes,
    App,
};
use futures::{executor::block_on, stream::Stream};
use serde_json::Value as JsonValue;

use super::settings::get_schema_registry_password;
use crate::db::DbAuxOperations;
use avro_schema_registry::app;
use avro_schema_registry::db::{DbConnection, DbManage, DbPool};

pub struct ApiTesterServer(test::TestServer);

pub fn setup() -> (ApiTesterServer, DbConnection) {
    let server = ApiTesterServer::new();
    let conn = DbPool::new_pool(Some(1)).connection().unwrap();
    conn.reset();

    (server, conn)
}

impl ApiTesterServer {
    pub fn new() -> Self {
        Self(test::start(|| {
            App::new()
                .configure(app::monitoring_routing)
                .data(DbPool::new_pool(Some(1)))
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
        self.header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::ACCEPT, "application/vnd.schemaregistry+json")
            .basic_auth("", Some(&get_schema_registry_password()))
    }
}

trait ValidateResponse {
    fn validate(self, expected_status: http::StatusCode, expected_body: &str);
}

impl<S> ValidateResponse for ClientResponse<S>
where
    S: Stream<Item = Result<Bytes, PayloadError>> + Unpin,
{
    fn validate(mut self, expected_status: http::StatusCode, expected_body: &str) {
        assert_eq!(self.status(), expected_status);
        let b = block_on(self.body()).unwrap();
        // TODO(nlopes): we should pass a Option instead of matching against empty string
        if expected_body != "" {
            assert_eq!(b, expected_body);
        }
    }
}

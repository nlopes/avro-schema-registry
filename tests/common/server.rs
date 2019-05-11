use actix_http::HttpService;
use actix_http_test::{TestServer, TestServerRuntime};
use actix_web::{
    client::{ClientRequest, ClientResponse},
    error::PayloadError,
    http, test,
    web::Bytes,
    App,
};
use futures::{future::Future, stream::Stream};
use serde_json::Value as JsonValue;

use super::settings::get_schema_registry_password;
use avro_schema_registry::app;
use avro_schema_registry::db::{DbManage, DbPool};

pub struct ApiTesterServer(TestServerRuntime);

impl ApiTesterServer {
    pub fn new() -> ApiTesterServer {
        ApiTesterServer(TestServer::new(|| {
            HttpService::new(
                App::new()
                    .configure(app::monitoring_routing)
                    .data(DbPool::new_pool(Some(1)))
                    .configure(app::api_routing),
            )
        }))
    }

    // TODO(nlopes): redo this whole mess
    pub fn test(
        &self,
        method: http::Method,
        path: &str,
        body: Option<JsonValue>,
        expected_status: http::StatusCode,
        expected_body: &str,
    ) {
        let ApiTesterServer(server) = self;
        let req = server.request(method, server.url(path)).avro_headers();

        match body {
            Some(b) => test::block_on(req.send_json(&b))
                .unwrap()
                .validate(expected_status, expected_body),

            None => test::block_on(req.send())
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
    S: Stream<Item = Bytes, Error = PayloadError>,
{
    fn validate(mut self, expected_status: http::StatusCode, expected_body: &str) {
        assert_eq!(self.status(), expected_status);
        let _ = self
            .body()
            .and_then(|b| {
                // TODO(nlopes): we should pass a Option instead of matching against empty string
                if expected_body != "" {
                    assert_eq!(b, expected_body);
                }
                Ok(())
            })
            .poll();
    }
}

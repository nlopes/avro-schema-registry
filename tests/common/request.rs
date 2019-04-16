use actix_web::client::{Client, ClientResponse};
use actix_web::{http, test};
use awc::error::PayloadError;
use bytes::Bytes;
use futures::{future::Future, stream::Stream};
use serde_json::Value as JsonValue;

use crate::common::settings::{get_host, get_port};

pub struct TestRequest<'a> {
    method: http::Method,
    path: &'a str,
    body: Option<JsonValue>,

    expect_status: Option<http::StatusCode>,
    expect_body: Option<&'a str>,
}

impl<'a> TestRequest<'a> {
    pub fn new(method: http::Method, path: &'a str, body: Option<JsonValue>) -> TestRequest<'a> {
        TestRequest {
            method,
            path,
            body,
            expect_status: None,
            expect_body: None,
        }
    }

    pub fn expects_status(mut self, expected_status: http::StatusCode) -> Self {
        self.expect_status = Some(expected_status);
        self
    }

    pub fn expects_body(mut self, expected_body: &'a str) -> Self {
        self.expect_body = Some(expected_body);
        self
    }

    pub fn assert(self) {
        let c = Client::build()
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::ACCEPT, "application/vnd.schemaregistry+json")
            .basic_auth("", Some("silly_password"))
            .finish()
            .request(
                self.method.clone(),
                format!("http://{}:{}{}", get_host(), get_port(), self.path),
            );

        match self.body.clone() {
            Some(b) => test::block_on(c.send_json(&b))
                .map_err(|e| panic!("Error: {:?}", e))
                .and_then(|response| {
                    if let (Some(st), Some(bd)) = (self.expect_status, self.expect_body) {
                        response.validate(st, bd);
                    } else {
                        panic!("Must provide expected status and body");
                    }
                    Ok(())
                })
                .unwrap(),
            None => test::block_on(c.send())
                .map_err(|e| panic!("Error: {:?}", e))
                .and_then(|response| {
                    if let (Some(st), Some(bd)) = (self.expect_status, self.expect_body) {
                        response.validate(st, bd);
                    } else {
                        panic!("Must provide expected status and body");
                    }
                    Ok(())
                })
                .unwrap(),
        };
    }
}

pub trait ValidateResponse {
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
                assert_eq!(b, expected_body);
                Ok(())
            })
            .poll();
    }
}

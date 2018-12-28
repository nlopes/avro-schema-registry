use actix_web::{error::Error, http, test};
use serde_json::Value as JsonValue;

pub struct ApiTestResponse {
    pub body: String,
    pub status: http::StatusCode,
}

pub fn make_request(
    srv: &mut test::TestServer,
    method: http::Method,
    path: &str,
    body: Option<JsonValue>,
) -> Result<ApiTestResponse, Error> {
    use actix_web::HttpMessage;
    use futures::future::Future;

    use super::avro::AvroHeaders;

    let request = match body {
        Some(b) => srv.client(method, path).with_avro_headers().json(b)?,
        None => srv.client(method, path).with_avro_headers().finish()?,
    };
    let response = srv.execute(request.send())?;
    let body = response.body().wait()?;

    Ok(ApiTestResponse {
        body: std::str::from_utf8(&body[..])?.to_string(),
        status: response.status(),
    })
}

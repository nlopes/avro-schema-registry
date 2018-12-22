use actix_web::error::{ErrorBadRequest, ParseError};
use actix_web::middleware::{Middleware, Started};
use actix_web::{HttpRequest, Result};

pub struct VerifyAcceptHeader;

impl<S> Middleware<S> for VerifyAcceptHeader {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let r = req.clone();
        let accept_content_type = r
            .headers()
            .get("Accept")
            .ok_or(ErrorBadRequest(ParseError::Header))?
            .to_str()
            .map_err(ErrorBadRequest)?;

        match accept_content_type {
            "application/vnd.schemaregistry+json" => Ok(Started::Done),
            "application/vnd.schemaregistry.v1+json" => Ok(Started::Done),
            "application/json" => Ok(Started::Done),
            _ => Err(ErrorBadRequest(ParseError::Header)),
        }
    }
}

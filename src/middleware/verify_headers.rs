use actix_http::error::*;
use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http,
};
use futures::future::{ok, Either, FutureResult};
use futures::{Future, Poll};

pub struct VerifyAcceptHeader;

const VALID_ACCEPT_HEADERS: [&str; 3] = [
    "application/vnd.schemaregistry+json",
    "application/vnd.schemaregistry.v1+json",
    "application/json",
];

impl VerifyAcceptHeader {
    fn is_valid(headers: &http::HeaderMap) -> bool {
        match headers.get(http::header::ACCEPT) {
            Some(v) => match v.to_str() {
                Ok(s) => VALID_ACCEPT_HEADERS.iter().any(|h| *h == s),
                _ => false,
            },
            None => false,
        }
    }
}

impl<S, B> Transform<S> for VerifyAcceptHeader
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type InitError = ();
    type Transform = VerifyAcceptHeaderMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(VerifyAcceptHeaderMiddleware { service })
    }
}

pub struct VerifyAcceptHeaderMiddleware<S> {
    service: S,
}

impl<S, B> Service for VerifyAcceptHeaderMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Either<
        FutureResult<Self::Response, Self::Error>,
        Box<Future<Item = Self::Response, Error = Self::Error>>,
    >;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if VerifyAcceptHeader::is_valid(req.headers()) {
            return Either::B(Box::new(self.service.call(req)));
        }
        Either::A(ok(req.error_response(ParseError::Header)))
    }
}

#[cfg(test)]
mod tests {
    use super::VerifyAcceptHeader;
    use actix_http::http::{header, HeaderMap, HeaderValue};

    #[test]
    fn middleware_accept_header_is_invalid() {
        let mut hm = HeaderMap::new();
        hm.insert(header::ACCEPT, HeaderValue::from_static("invalid"));
        assert!(!VerifyAcceptHeader::is_valid(&hm));
    }

    #[test]
    fn middleware_accept_header_missing() {
        let hm = HeaderMap::new();
        assert!(!VerifyAcceptHeader::is_valid(&hm));
    }

    #[test]
    fn middleware_accept_header_is_valid() {
        let mut hm = HeaderMap::new();
        hm.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        assert!(VerifyAcceptHeader::is_valid(&hm));
    }
}

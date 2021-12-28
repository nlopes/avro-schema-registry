use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ParseError,
    http,
};
use futures::future::{ok, Either, Ready};
use futures::task::{Context, Poll};

pub struct VerifyAcceptHeader;

const VALID_ACCEPT_HEADERS: [&str; 3] = [
    "application/vnd.schemaregistry+json",
    "application/vnd.schemaregistry.v1+json",
    "application/json",
];

impl VerifyAcceptHeader {
    fn is_valid(headers: &http::header::HeaderMap) -> bool {
        match headers.get(http::header::ACCEPT) {
            Some(v) => match v.to_str() {
                Ok(s) => VALID_ACCEPT_HEADERS.iter().any(|h| *h == s),
                _ => false,
            },
            None => false,
        }
    }
}

impl<S> Transform<S, ServiceRequest> for VerifyAcceptHeader
where
    S: Service<ServiceRequest, Response = ServiceResponse>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = S::Error;
    type InitError = ();
    type Transform = VerifyAcceptHeaderMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(VerifyAcceptHeaderMiddleware { service })
    }
}

pub struct VerifyAcceptHeaderMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for VerifyAcceptHeaderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = S::Error;
    type Future = Either<Ready<Result<Self::Response, Self::Error>>, S::Future>;

    fn poll_ready(&self, ct: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ct)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if VerifyAcceptHeader::is_valid(req.headers()) {
            return Either::Right(self.service.call(req));
        }
        Either::Left(ok(req.error_response(ParseError::Header)))
    }
}

#[cfg(test)]
mod tests {
    use super::VerifyAcceptHeader;
    use actix_web::http::header::{self, HeaderMap, HeaderValue};

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

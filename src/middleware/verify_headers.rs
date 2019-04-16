use actix_http::error::*;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use futures::future::{ok, Either, FutureResult};
use futures::{Future, Poll};

pub struct VerifyAcceptHeader;

//const VALID_ACCEPT_HEADERS: [&str] = ["app", "meh", "doh"];

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
        let headers = req.headers();
        match headers.get("Accept") {
            Some(v) => match v.to_str() {
                Ok("application/vnd.schemaregistry+json")
                | Ok("application/vnd.schemaregistry.v1+json")
                | Ok("application/json") => Either::B(Box::new(self.service.call(req))),
                _ => Either::A(ok(req.error_response(ParseError::Header))),
            },
            None => Either::A(ok(req.error_response(ParseError::Header))),
        }
    }
}

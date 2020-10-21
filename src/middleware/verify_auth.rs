use actix_http::{error::*, http::HeaderMap};
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use futures::future::{ok, Either, Ready};
use futures::task::{Context, Poll};

pub struct VerifyAuthorization {
    password: String,
}

impl VerifyAuthorization {
    pub fn new(password: &str) -> Self {
        Self {
            password: password.to_string(),
        }
    }

    fn validate(headers: &HeaderMap, password: &str) -> Result<(), Error> {
        let authorization = headers
            .get("Authorization")
            .ok_or_else(|| ErrorBadRequest(ParseError::Header))?
            .to_str()
            .map_err(ErrorBadRequest)?;

        if authorization.len() < 7 {
            // 'Basic ' is 6 chars long, so anything below 7 is invalid
            return Err(ErrorBadRequest(ParseError::Header));
        }

        let (basic, base64_auth) = authorization.split_at(6);
        if basic.ne("Basic ") {
            return Err(ErrorBadRequest(ParseError::Header));
        }

        match base64::decode(base64_auth) {
            Ok(bytes) => {
                let mut basic_creds = std::str::from_utf8(&bytes)?
                    .trim_end_matches('\n')
                    .splitn(2, ':');
                let _username = basic_creds
                    .next()
                    .ok_or_else(|| ErrorBadRequest(ParseError::Header))?;

                let header_password = basic_creds
                    .next()
                    .ok_or_else(|| ErrorBadRequest(ParseError::Header))?;

                if *header_password != *password {
                    return Err(ErrorForbidden(ParseError::Header));
                }
                Ok(())
            }
            Err(_) => Err(ErrorBadRequest(ParseError::Header)),
        }
    }
}

impl<S> Transform<S> for VerifyAuthorization
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = S::Error;
    type InitError = ();
    type Transform = VerifyAuthorizationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(VerifyAuthorizationMiddleware {
            service,
            password: self.password.clone(),
        })
    }
}

pub struct VerifyAuthorizationMiddleware<S> {
    service: S,
    password: String,
}

impl<S> Service for VerifyAuthorizationMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = S::Error;
    type Future = Either<Ready<Result<Self::Response, Self::Error>>, S::Future>;

    fn poll_ready(&mut self, ct: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ct)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        match VerifyAuthorization::validate(req.headers(), &self.password) {
            Ok(_) => Either::Right(self.service.call(req)),
            Err(_) => Either::Left(ok(req.error_response(ParseError::Header))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VerifyAuthorization;
    use actix_http::http::{header, HeaderMap, HeaderValue};

    const VALID_PASSWORD: &str = "some_password";
    const INVALID_PASSWORD: &str = "some_invalid_password";
    const CORRECT_AUTH: &str = "Basic OnNvbWVfcGFzc3dvcmQK";

    #[test]
    fn middleware_with_valid_password() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static(CORRECT_AUTH),
        );
        assert!(VerifyAuthorization::validate(&headers, VALID_PASSWORD).is_ok());
    }

    #[test]
    fn middleware_with_invalid_password() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static(CORRECT_AUTH),
        );
        assert!(VerifyAuthorization::validate(&headers, INVALID_PASSWORD).is_err());
    }

    #[test]
    fn middleware_with_malformed_header() {
        let headers = HeaderMap::new();
        assert!(VerifyAuthorization::validate(&headers, VALID_PASSWORD).is_err());
    }

    #[test]
    fn middleware_with_malformed_header_content() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("bad"));
        assert!(VerifyAuthorization::validate(&headers, VALID_PASSWORD).is_err());
    }

    #[test]
    fn middleware_with_wrong_content_length() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic "));
        assert!(VerifyAuthorization::validate(&headers, VALID_PASSWORD).is_err());
    }

    #[test]
    fn middleware_with_bad_base64() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic meh"));
        assert!(VerifyAuthorization::validate(&headers, VALID_PASSWORD).is_err());
    }
}

use actix_web::error::{Error, ErrorBadRequest, ErrorForbidden, ParseError};
use actix_web::middleware::Middleware;
use actix_web::middleware::Started;
use actix_web::{HttpRequest, Request, Result};

use base64;

#[derive(Default)]
pub struct VerifyAuthorization {
    password: String,
}

impl VerifyAuthorization {
    pub fn new(password: &String) -> VerifyAuthorization {
        VerifyAuthorization {
            password: password.to_string(),
        }
    }

    fn validate(&self, req: &Request) -> Result<(), Error> {
        let authorization = req
            .headers()
            .get("Authorization")
            .ok_or(ErrorBadRequest(ParseError::Header))?
            .to_str()
            .map_err(ErrorBadRequest)?;

        if authorization.len() < 7 {
            // 'Basic ' is 6 chars long, so anything below 7 is invalid
            return Err(ErrorBadRequest(ParseError::Header));
        }

        //TODO: is it worth checking basic matches "Basic"? I don't think so
        let (_basic, base64_auth) = authorization.split_at(6);

        match base64::decode(base64_auth) {
            Ok(bytes) => {
                let mut basic_creds = std::str::from_utf8(&bytes)?.splitn(2, ':');
                let _username = basic_creds
                    .next()
                    .ok_or(ErrorBadRequest(ParseError::Header))
                    .map(|username| username.to_string())?;

                let password = basic_creds
                    .next()
                    .ok_or(ErrorBadRequest(ParseError::Header))
                    .map(|password| password.to_string())?;

                if password[..password.len() - 1] != self.password {
                    return Err(ErrorForbidden(ParseError::Header));
                }
                Ok(())
            }
            Err(_) => Err(ErrorBadRequest(ParseError::Header)),
        }
    }
}

// TODO: this middleware is right now a cluster fuck. Disgusting Norberto, disgusting.
impl<S> Middleware<S> for VerifyAuthorization {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        self.validate(&req)?;
        Ok(Started::Done)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::Method;
    use actix_web::test::TestRequest;

    #[test]
    fn middleware_with_valid_password() {
        let password = "some_password";
        let auth = VerifyAuthorization::new(&password.to_string());

        let req = TestRequest::with_header("Authorization", "Basic OnNvbWVfcGFzc3dvcmQK")
            .method(Method::HEAD)
            .finish();

        assert!(auth.start(&req).is_ok());
    }

    #[test]
    fn middleware_with_invalid_password() {
        let password = "some_invalid_password";
        let auth = VerifyAuthorization::new(&password.to_string());
        let req = TestRequest::with_header("Authorization", "Basic OnNvbWVfcGFzc3dvcmQK")
            .method(Method::HEAD)
            .finish();

        assert!(auth.start(&req).is_err());
    }

    #[test]
    fn middleware_with_malformed_header() {
        let password = "some_password";
        let auth = VerifyAuthorization::new(&password.to_string());

        let req = TestRequest::with_header("Authy", "bad")
            .method(Method::HEAD)
            .finish();

        assert!(auth.start(&req).is_err());
    }

    #[test]
    fn middleware_with_malformed_header_content() {
        let password = "some_password";
        let auth = VerifyAuthorization::new(&password.to_string());

        let req = TestRequest::with_header("Authorization", "bad")
            .method(Method::HEAD)
            .finish();

        assert!(auth.start(&req).is_err());
    }

    #[test]
    fn middleware_with_wrong_content_length() {
        let password = "some_password";
        let auth = VerifyAuthorization::new(&password.to_string());

        let req = TestRequest::with_header("Authorization", "Basic ")
            .method(Method::HEAD)
            .finish();

        assert!(auth.start(&req).is_err());
    }

    #[test]
    fn middleware_with_bad_base64() {
        let password = "some_password";
        let auth = VerifyAuthorization::new(&password.to_string());

        let req = TestRequest::with_header("Authorization", "Basic meh")
            .method(Method::HEAD)
            .finish();

        assert!(auth.start(&req).is_err());
    }
}

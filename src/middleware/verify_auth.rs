use actix_web::error::{ErrorBadRequest, ErrorForbidden, ParseError};
use actix_web::middleware::{Middleware, Started};
use actix_web::{HttpRequest, Result};

use base64;

pub struct VerifyAuthorization {
    password: String,
}

impl VerifyAuthorization {
    pub fn new(password: &String) -> VerifyAuthorization {
        VerifyAuthorization {
            password: password.to_string(),
        }
    }
}

// TODO: this middleware is right now a cluster fuck. Disgusting Norberto, disgusting.
impl<S> Middleware<S> for VerifyAuthorization {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let r = req.clone();
        let authorization = r
            .headers()
            .get("Authorization")
            .ok_or(ErrorBadRequest(ParseError::Header))?
            .to_str()
            .map_err(ErrorBadRequest)?;

        if authorization.len() < 7 {
            // Basic C
            return Err(ErrorBadRequest(ParseError::Header));
        }
        let (_basic, base64_auth) = authorization.split_at(6);
        //TODO: is it worth checking basic matches "Basic"? I don't think so

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

                // TODO: For some reason base64::decode (I think) injects a newline at the
                // end, so we need to remove it before we compare
                if password[..password.len() - 1] != self.password {
                    return Err(ErrorForbidden(ParseError::Header));
                }
                Ok(Started::Done)
            }
            Err(_) => Err(ErrorBadRequest(ParseError::Header)),
        }
    }
}

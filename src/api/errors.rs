use actix_web::{http::StatusCode, HttpResponse};

#[derive(Serialize)]
pub enum ApiStatusCode {
    NotFound = 404,
    Conflict = 409,

    UnprocessableEntity = 422,

    InternalServerError = 500,
}

// TODO: maybe replace this with serde_aux::serde_aux_enum_number_declare
macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_u16(*self as u16)
            }
        }
    }
}

// We use the macro to ensure we serialize as numbers, not as the name.
enum_number!(ApiErrorCode {
    SubjectNotFound = 40401,
    VersionNotFound = 40402,
    SchemaNotFound = 40403,

    InvalidAvroSchema = 42201,
    InvalidVersion = 42202,
    InvalidCompatibilityLevel = 42203,

    BackendDatastoreError = 50001,
    OperationTimedOut = 50002,
    MasterForwardingError = 50003,
});

impl ApiErrorCode {
    pub fn message(&self) -> &str {
        match self {
            ApiErrorCode::SubjectNotFound => "Subject not found",
            ApiErrorCode::VersionNotFound => "Version not found",
            ApiErrorCode::SchemaNotFound => "Schema not found",

            ApiErrorCode::InvalidAvroSchema => "Invalid Avro schema",
            ApiErrorCode::InvalidVersion => "Invalid version",
            ApiErrorCode::InvalidCompatibilityLevel => "Invalid compatibility level",

            ApiErrorCode::BackendDatastoreError => "Error in the backend datastore",
            ApiErrorCode::OperationTimedOut => "Operation timed out",
            ApiErrorCode::MasterForwardingError => {
                "Error while forwarding the request to the master"
            }
        }
    }
}

impl std::fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.message())
    }
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub error_code: ApiErrorCode,
    pub message: String,
}

#[derive(Serialize)]
pub struct ApiError {
    pub status_code: ApiStatusCode,
    pub response: ApiErrorResponse,
}

impl ApiError {
    pub fn new(error_code: ApiErrorCode) -> Self {
        let status_code = match error_code {
            ApiErrorCode::SubjectNotFound => ApiStatusCode::NotFound,
            ApiErrorCode::VersionNotFound => ApiStatusCode::NotFound,
            ApiErrorCode::SchemaNotFound => ApiStatusCode::NotFound,

            ApiErrorCode::InvalidAvroSchema => ApiStatusCode::UnprocessableEntity,
            ApiErrorCode::InvalidVersion => ApiStatusCode::UnprocessableEntity,
            ApiErrorCode::InvalidCompatibilityLevel => ApiStatusCode::UnprocessableEntity,

            ApiErrorCode::BackendDatastoreError => ApiStatusCode::InternalServerError,
            ApiErrorCode::OperationTimedOut => ApiStatusCode::InternalServerError,
            ApiErrorCode::MasterForwardingError => ApiStatusCode::InternalServerError,
        };
        Self::with_optional_message(status_code, error_code, None)
    }

    pub fn with_message(
        status_code: ApiStatusCode,
        error_code: ApiErrorCode,
        message: String,
    ) -> Self {
        Self::with_optional_message(status_code, error_code, Some(message))
    }

    fn with_optional_message(
        status_code: ApiStatusCode,
        error_code: ApiErrorCode,
        message: Option<String>,
    ) -> Self {
        ApiError {
            status_code: status_code,
            response: ApiErrorResponse {
                error_code: error_code,
                message: if let Some(extra) = message {
                    format!("{}: {}", error_code.message(), extra)
                } else {
                    error_code.message().to_string()
                },
            },
        }
    }

    pub fn http_response(&self) -> HttpResponse {
        // TODO: do this in a better way, I shouldn't need to call a function for this. What I
        // should do instead is implement a trait on FutureResponse<HttpResponse>
        match self.status_code {
            ApiStatusCode::NotFound => HttpResponse::NotFound().json(&self.response),
            ApiStatusCode::InternalServerError => {
                HttpResponse::InternalServerError().json(&self.response)
            }
            ApiStatusCode::UnprocessableEntity => {
                HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).json(&self.response)
            }
            _ => HttpResponse::NotImplemented().finish(),
        }
    }
}

impl std::error::Error for ApiErrorCode {
    fn description(&self) -> &str {
        self.message()
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl std::convert::From<diesel::result::Error> for ApiErrorCode {
    fn from(_error: diesel::result::Error) -> Self {
        ApiErrorCode::BackendDatastoreError
    }
}

impl std::convert::From<diesel::result::Error> for ApiError {
    fn from(_error: diesel::result::Error) -> Self {
        ApiError::new(ApiErrorCode::BackendDatastoreError)
    }
}

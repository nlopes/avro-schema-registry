use actix;
use actix_web::{http::StatusCode, HttpResponse};
use serde_json;

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
enum_number!(ApiAvroErrorCode {
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

impl ApiAvroErrorCode {
    pub fn message(&self) -> &str {
        match self {
            ApiAvroErrorCode::SubjectNotFound => "Subject not found",
            ApiAvroErrorCode::VersionNotFound => "Version not found",
            ApiAvroErrorCode::SchemaNotFound => "Schema not found",

            ApiAvroErrorCode::InvalidAvroSchema => "Invalid Avro schema",
            ApiAvroErrorCode::InvalidVersion => "Invalid version",
            ApiAvroErrorCode::InvalidCompatibilityLevel => "Invalid compatibility level",

            ApiAvroErrorCode::BackendDatastoreError => "Error in the backend datastore",
            ApiAvroErrorCode::OperationTimedOut => "Operation timed out",
            ApiAvroErrorCode::MasterForwardingError => {
                "Error while forwarding the request to the master"
            }
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ApiErrorResponse {
    pub error_code: ApiAvroErrorCode,
    pub message: String,
}

#[derive(Debug, Fail, Clone)]
#[fail(display = "{}", response)]
pub struct ApiError {
    pub status_code: StatusCode,
    pub response: ApiErrorResponse,
}

impl ApiError {
    pub fn new(error_code: ApiAvroErrorCode) -> Self {
        let status_code = match error_code {
            ApiAvroErrorCode::SubjectNotFound => StatusCode::NOT_FOUND,
            ApiAvroErrorCode::VersionNotFound => StatusCode::NOT_FOUND,
            ApiAvroErrorCode::SchemaNotFound => StatusCode::NOT_FOUND,

            ApiAvroErrorCode::InvalidAvroSchema => StatusCode::UNPROCESSABLE_ENTITY,
            ApiAvroErrorCode::InvalidVersion => StatusCode::UNPROCESSABLE_ENTITY,
            ApiAvroErrorCode::InvalidCompatibilityLevel => StatusCode::UNPROCESSABLE_ENTITY,

            ApiAvroErrorCode::BackendDatastoreError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiAvroErrorCode::OperationTimedOut => StatusCode::INTERNAL_SERVER_ERROR,
            ApiAvroErrorCode::MasterForwardingError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiError {
            status_code,
            response: ApiErrorResponse {
                error_code,
                message: error_code.message().to_string(),
            },
        }
    }
}

impl std::fmt::Display for ApiAvroErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.message())
    }
}

impl std::fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::json!(self))
    }
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self.status_code {
            StatusCode::NOT_FOUND => HttpResponse::NotFound().json(&self.response),
            StatusCode::INTERNAL_SERVER_ERROR => {
                HttpResponse::InternalServerError().json(&self.response)
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).json(&self.response)
            }
            _ => HttpResponse::NotImplemented().finish(),
        }
    }
}

impl std::error::Error for ApiAvroErrorCode {
    fn description(&self) -> &str {
        self.message()
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl std::convert::From<diesel::result::Error> for ApiAvroErrorCode {
    fn from(_error: diesel::result::Error) -> Self {
        ApiAvroErrorCode::BackendDatastoreError
    }
}

impl std::convert::From<diesel::result::Error> for ApiError {
    fn from(_error: diesel::result::Error) -> Self {
        ApiError::new(ApiAvroErrorCode::BackendDatastoreError)
    }
}

impl std::convert::From<actix::MailboxError> for ApiError {
    fn from(_error: actix::MailboxError) -> Self {
        ApiError::new(ApiAvroErrorCode::BackendDatastoreError)
    }
}

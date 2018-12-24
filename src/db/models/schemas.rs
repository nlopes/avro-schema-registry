use actix::Message;

extern crate chrono;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::api::errors::{ApiError, ApiErrorCode};

use super::schema::*;

#[derive(Debug, Identifiable, Associations, Queryable)]
#[table_name = "schemas"]
pub struct Schema {
    pub id: i64,
    pub fingerprint: String,
    pub json: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub fingerprint2: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SchemaResponse {
    pub schema: String,
}

pub struct GetSchema {
    pub id: i64,
}

impl Message for GetSchema {
    type Result = Result<SchemaResponse, ApiError>;
}

impl Schema {
    // TODO: it's a bit crap to pre-optimise on this select. This function name smells too.
    pub fn get_json_by_id(schema_id: i64, conn: &PgConnection) -> Result<String, ApiError> {
        use super::schema::schemas::dsl::*;
        schemas
            .find(schema_id)
            .select(json)
            .get_result::<String>(conn)
            .or(Err(ApiError::new(ApiErrorCode::SchemaNotFound)))
    }
}

pub struct DeleteSchemaVersion {
    pub subject: String,
    pub version: i32,
}

impl Message for DeleteSchemaVersion {
    type Result = Result<i32, ApiError>;
}

impl DeleteSchemaVersion {
    pub fn execute(&self, conn: &PgConnection) -> Result<i32, ApiError> {
        use super::{SchemaVersion, Subject};

        use super::schema::schema_versions::dsl::version;
        use super::schema::subjects::dsl::{name, subjects};

        conn.transaction::<_, ApiError, _>(|| {
            subjects
                .filter(name.eq(self.subject.to_string()))
                .get_result::<Subject>(conn)
                .or_else(|_| Err(ApiError::new(ApiErrorCode::SubjectNotFound)))
                .and_then(|subject| {
                    diesel::delete(
                        SchemaVersion::belonging_to(&subject).filter(version.eq(self.version)),
                    )
                    .execute(conn)
                    .or_else(|_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)))
                    .and_then(|o| match o {
                        0 => Err(ApiError::new(ApiErrorCode::VersionNotFound)),
                        _ => Ok(self.version),
                    })
                })
        })
    }
}

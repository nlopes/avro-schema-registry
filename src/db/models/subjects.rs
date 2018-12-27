use actix::Message;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use super::schema::*;
use super::SchemaVersion;

use crate::api::errors::{ApiError, ApiErrorCode};

#[derive(Debug, Identifiable, Associations, Queryable, Serialize)]
#[table_name = "subjects"]
pub struct Subject {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Subject {
    /// Insert a new subject but ignore if it already exists.
    ///
    /// *Note:* 'ignore' in the case above means we will update the name if it already
    /// exists. This spares us complicated code to fetch, verify and then insert.
    pub fn insert(conn: &PgConnection, subject: String) -> Result<Subject, ApiError> {
        use super::schema::subjects::dsl::*;

        diesel::insert_into(subjects)
            .values((
                name.eq(&subject),
                created_at.eq(diesel::dsl::now),
                updated_at.eq(diesel::dsl::now),
            ))
            .on_conflict(name)
            .do_update()
            .set(name.eq(&subject))
            .get_result::<Subject>(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    pub fn get_by_name(conn: &PgConnection, subject: String) -> Result<Self, ApiError> {
        use super::schema::subjects::dsl::{name, subjects};
        match subjects.filter(name.eq(subject)).first::<Subject>(conn) {
            Ok(s) => Ok(s),
            Err(diesel::result::Error::NotFound) => {
                Err(ApiError::new(ApiErrorCode::SubjectNotFound))
            }
            _ => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubjectList {
    pub content: Vec<String>,
}

pub struct GetSubjects;

impl Message for GetSubjects {
    type Result = Result<SubjectList, ApiError>;
}

#[derive(Debug, Serialize)]
pub struct SubjectVersionsResponse {
    // TODO: this should be a new type with values between 1 and 2^31-1
    pub versions: Vec<Option<i32>>,
}

pub struct GetSubjectVersions {
    pub subject: String,
}

impl Message for GetSubjectVersions {
    type Result = Result<SubjectVersionsResponse, ApiError>;
}

#[derive(Debug, Serialize)]
pub struct DeleteSubjectResponse {
    pub versions: Vec<Option<i32>>,
}

pub struct DeleteSubject {
    pub subject: String,
}

impl DeleteSubject {
    pub fn delete(
        subject_name: String,
        conn: &PgConnection,
    ) -> Result<DeleteSubjectResponse, ApiError> {
        use super::SchemaVersion;

        SchemaVersion::delete_subject_with_name(subject_name, &conn).map_or_else(
            |_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
            |res| {
                if res.len() != 0 {
                    Ok(DeleteSubjectResponse { versions: res })
                } else {
                    Err(ApiError::new(ApiErrorCode::SubjectNotFound))
                }
            },
        )
    }
}

impl Message for DeleteSubject {
    type Result = Result<DeleteSubjectResponse, ApiError>;
}

#[derive(Debug, Serialize)]
pub struct GetSubjectVersion {
    pub subject: String,
    pub version: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct GetSubjectVersionResponse {
    pub subject: String,
    // TODO: The documentation mentions this but their example response doesn't include it
    pub id: i64,
    pub version: i32,
    pub schema: String,
}

impl GetSubjectVersion {
    pub fn execute(&self, conn: &PgConnection) -> Result<GetSubjectVersionResponse, ApiError> {
        match self.version {
            Some(v) => SchemaVersion::get_schema_id(self.subject.to_string(), v as i32, conn),
            None => SchemaVersion::get_schema_id_from_latest(self.subject.to_string(), conn),
        }
        .map_or_else(
            |e| Err(e),
            |o| {
                Ok(GetSubjectVersionResponse {
                    subject: self.subject.to_string(),
                    id: o.0,
                    version: o.1,
                    schema: o.2,
                })
            },
        )
    }
}

impl Message for GetSubjectVersion {
    type Result = Result<GetSubjectVersionResponse, ApiError>;
}

pub struct VerifySchemaRegistration {
    pub subject: String,
    pub schema: String,
}

impl VerifySchemaRegistration {
    pub fn execute(&self, conn: &PgConnection) -> Result<GetSubjectVersionResponse, ApiError> {
        use super::schema::schema_versions::dsl::{
            schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{id as schemas_id, json, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        conn.transaction::<_, ApiError, _>(|| {
            subjects
                .filter(name.eq(self.subject.to_string()))
                .select(subjects_id)
                .get_result(conn)
                .or_else(|_| Err(ApiError::new(ApiErrorCode::SubjectNotFound)))
                .and_then(|subject_id_found: i64| {
                    schemas
                        .filter(json.eq(self.schema.to_string()))
                        .select(schemas_id)
                        .get_result(conn)
                        .or_else(|_| Err(ApiError::new(ApiErrorCode::SchemaNotFound)))
                        .and_then(|schema_id_found| {
                            schema_versions
                                .filter(subject_id.eq(subject_id_found))
                                .filter(schema_id.eq(schema_id_found))
                                .select(version)
                                .get_result(conn)
                                .or_else(|_| Err(ApiError::new(ApiErrorCode::VersionNotFound)))
                                .and_then(|schema_version_version: Option<i32>| {
                                    Ok(GetSubjectVersionResponse {
                                        subject: self.subject.to_string(),
                                        id: schema_id_found,
                                        version: schema_version_version.ok_or_else(|| {
                                            ApiError::new(ApiErrorCode::VersionNotFound)
                                        })?,
                                        schema: self.schema.to_string(),
                                    })
                                })
                        })
                })
        })
        .map_or_else(|e| Err(e), |x| Ok(x))
    }
}

impl Message for VerifySchemaRegistration {
    type Result = Result<GetSubjectVersionResponse, ApiError>;
}

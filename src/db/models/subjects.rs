use actix::Message;
use avro_rs;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use sha2::Sha256;

use super::schema::*;
use super::Schema;
use super::SchemaVersion;

use crate::api::errors::{ApiError, ApiErrorCode};

#[derive(Debug, Insertable, Identifiable, Associations, Queryable, Serialize)]
#[table_name = "subjects"]
pub struct Subject {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Subject {
    pub fn get_by_name(subject: String, conn: &PgConnection) -> Result<Self, ApiError> {
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

#[derive(Debug, Serialize)]
pub struct RegisterSchemaResponse {
    pub id: String,
}

pub struct RegisterSchema {
    pub subject: String,
    pub schema: String,
}

impl RegisterSchema {
    pub fn parse(json: String) -> Result<avro_rs::Schema, ApiError> {
        avro_rs::Schema::parse_str(&json)
            .map_err(|_| ApiError::new(ApiErrorCode::InvalidAvroSchema))
    }

    fn generate_fingerprint(&self) -> Result<String, ApiError> {
        Ok(format!(
            "{}",
            RegisterSchema::parse(self.schema.to_owned())?.fingerprint::<Sha256>()
        ))
    }

    pub fn find_schema(&self, conn: &PgConnection) -> Result<Option<Schema>, ApiError> {
        use super::schema::schemas::dsl::{fingerprint2, schemas};
        let fingerprint = self.generate_fingerprint()?;
        Ok(schemas
            .filter(fingerprint2.eq(fingerprint))
            .load::<Schema>(conn)
            .map_err(|e| {
                println!("{}", e);
                ApiError::new(ApiErrorCode::BackendDatastoreError)
            })?
            .pop())
    }

    pub fn create_new_schema(&self, conn: &PgConnection) -> Result<Subject, ApiError> {
        use super::schema::schemas::dsl::{
            created_at as schema_created_at, fingerprint, fingerprint2, json, schemas,
            updated_at as schema_updated_at,
        };
        use super::schema::subjects::dsl::{
            created_at as subject_created_at, name, subjects, updated_at as subject_updated_at,
        };

        use super::schema::schema_versions::dsl::{schema_id, schema_versions, subject_id};

        // TODO: we use the same in both fields. This means we don't do the same as
        // salsify
        let schema_fingerprint = self.generate_fingerprint()?;

        conn.transaction::<_, diesel::result::Error, _>(|| {
            diesel::insert_into(schemas)
                .values((
                    json.eq(self.schema.to_owned()),
                    fingerprint.eq(schema_fingerprint.to_owned()),
                    fingerprint2.eq(schema_fingerprint),
                    schema_created_at.eq(diesel::dsl::now),
                    schema_updated_at.eq(diesel::dsl::now),
                ))
                .get_result::<Schema>(conn)
                .and_then(|schema| {
                    diesel::insert_into(subjects)
                        .values((
                            name.eq(self.subject.to_owned()),
                            subject_created_at.eq(diesel::dsl::now),
                            subject_updated_at.eq(diesel::dsl::now),
                        ))
                        .get_result::<Subject>(conn)
                        .and_then(|subject| {
                            diesel::insert_into(schema_versions)
                                .values((subject_id.eq(subject.id), schema_id.eq(schema.id)))
                                .execute(conn)
                                .and(Ok(subject))
                        })
                })
        })
        .map_or_else(
            |_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
            |x| Ok(x),
        )
    }
}

impl Message for RegisterSchema {
    type Result = Result<RegisterSchemaResponse, ApiError>;
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

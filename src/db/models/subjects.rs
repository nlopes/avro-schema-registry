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
//#[has_many(SchemaVersions)]
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

impl Message for DeleteSubject {
    type Result = Result<DeleteSubjectResponse, ApiError>;
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
            let schema = diesel::insert_into(schemas)
                .values((
                    json.eq(self.schema.to_owned()),
                    fingerprint.eq(schema_fingerprint.to_owned()),
                    fingerprint2.eq(schema_fingerprint),
                    schema_created_at.eq(diesel::dsl::now),
                    schema_updated_at.eq(diesel::dsl::now),
                ))
                .load::<Schema>(conn)?
                .pop()
                .ok_or(diesel::result::Error::NotFound)?;

            let subject = diesel::insert_into(subjects)
                .values((
                    name.eq(self.subject.to_owned()),
                    subject_created_at.eq(diesel::dsl::now),
                    subject_updated_at.eq(diesel::dsl::now),
                ))
                .load::<Subject>(conn)?
                .pop()
                .ok_or(diesel::result::Error::NotFound)?;

            let schema_version = diesel::insert_into(schema_versions)
                .values((subject_id.eq(subject.id), schema_id.eq(schema.id)))
                .load::<SchemaVersion>(conn)?
                .pop()
                .ok_or(diesel::result::Error::NotFound);

            Ok(subject)
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

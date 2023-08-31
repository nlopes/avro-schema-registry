use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

use super::schema::*;

use crate::api::errors::{ApiAvroErrorCode, ApiError};

#[derive(Debug, Identifiable, Queryable, Serialize)]
#[diesel(table_name = subjects)]
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
    pub fn insert(conn: &mut PgConnection, subject: String) -> Result<Self, ApiError> {
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
            .get_result::<Self>(conn)
            .map_err(|_| ApiError::new(ApiAvroErrorCode::BackendDatastoreError))
    }

    pub fn distinct_names(conn: &mut PgConnection) -> Result<Vec<String>, ApiError> {
        use super::schema::subjects::dsl::{name, subjects};

        subjects
            .select(name)
            .load::<String>(conn)
            .map_err(|_| ApiError::new(ApiAvroErrorCode::BackendDatastoreError))
    }

    pub fn get_by_name(conn: &mut PgConnection, subject: String) -> Result<Self, ApiError> {
        use super::schema::subjects::dsl::{name, subjects};
        match subjects.filter(name.eq(subject)).first::<Self>(conn) {
            Ok(s) => Ok(s),
            Err(diesel::result::Error::NotFound) => {
                Err(ApiError::new(ApiAvroErrorCode::SubjectNotFound))
            }
            _ => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
        }
    }

    pub fn delete_by_name(
        conn: &mut PgConnection,
        subject_name: String,
    ) -> Result<Vec<Option<i32>>, ApiError> {
        use super::SchemaVersion;

        match SchemaVersion::delete_subject_with_name(conn, subject_name) {
            Err(_) => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
            Ok(res) => {
                if !res.is_empty() {
                    Ok(res)
                } else {
                    Err(ApiError::new(ApiAvroErrorCode::SubjectNotFound))
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubjectList {
    pub content: Vec<String>,
}

pub struct GetSubjects;

#[derive(Debug, Serialize)]
pub struct SubjectVersionsResponse {
    // TODO: this should be a new type with values between 1 and 2^31-1
    pub versions: Vec<Option<i32>>,
}

pub struct GetSubjectVersions {
    pub subject: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteSubjectResponse {
    pub versions: Vec<Option<i32>>,
}

pub struct DeleteSubject {
    pub subject: String,
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

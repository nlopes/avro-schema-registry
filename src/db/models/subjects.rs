use actix::Message;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use super::schema::*;

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

    pub fn distinct_names(conn: &PgConnection) -> Result<Vec<String>, ApiError> {
        use super::schema::subjects::dsl::{name, subjects};

        subjects
            .select(name)
            .load::<String>(conn)
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

    pub fn delete_by_name(
        conn: &PgConnection,
        subject_name: String,
    ) -> Result<Vec<Option<i32>>, ApiError> {
        use super::SchemaVersion;

        SchemaVersion::delete_subject_with_name(&conn, subject_name).map_or_else(
            |_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
            |res| {
                if res.len() != 0 {
                    Ok(res)
                } else {
                    Err(ApiError::new(ApiErrorCode::SubjectNotFound))
                }
            },
        )
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

impl Message for GetSubjectVersion {
    type Result = Result<GetSubjectVersionResponse, ApiError>;
}

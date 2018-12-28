use actix::Message;
use diesel::prelude::*;

use super::schema::*;
use super::schemas::Schema;
use super::subjects::Subject;

use crate::api::errors::{ApiError, ApiErrorCode};

#[derive(Debug, Identifiable, Associations, Queryable)]
#[table_name = "schema_versions"]
#[belongs_to(Schema)]
#[belongs_to(Subject)]
pub struct SchemaVersion {
    pub id: i64,
    pub version: Option<i32>,
    pub subject_id: i64,
    pub schema_id: i64,
}

#[derive(Debug, Insertable)]
#[table_name = "schema_versions"]
pub struct NewSchemaVersion {
    pub version: Option<i32>,
    pub subject_id: i64,
    pub schema_id: i64,
}

pub type SchemaVersionFields = NewSchemaVersion;

impl SchemaVersion {
    pub fn insert(conn: &PgConnection, sv: NewSchemaVersion) -> Result<SchemaVersion, ApiError> {
        use super::schema::schema_versions::dsl::schema_versions;
        diesel::insert_into(schema_versions)
            .values(&sv)
            .get_result::<SchemaVersion>(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    pub fn find(
        conn: &PgConnection,
        find_subject_id: i64,
        find_schema_id: i64,
    ) -> Result<SchemaVersion, ApiError> {
        use super::schema::schema_versions::dsl::{schema_id, schema_versions, subject_id};

        schema_versions
            .filter(subject_id.eq(find_subject_id))
            .filter(schema_id.eq(find_schema_id))
            .get_result::<SchemaVersion>(conn)
            .or_else(|_| Err(ApiError::new(ApiErrorCode::VersionNotFound)))
    }

    pub fn with_schema_and_subject(
        conn: &PgConnection,
        search_subject_name: String,
        search_schema_id: i64,
    ) -> Result<usize, ApiError> {
        use super::schema::schema_versions::dsl::{id, schema_id, schema_versions, subject_id};
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, name as subject_name, subjects};

        schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .inner_join(schemas.on(schema_id.eq(schemas_id)))
            .filter(subject_name.eq(search_subject_name))
            .filter(schema_id.eq(search_schema_id))
            .select(id)
            .execute(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    pub fn versions_with_subject_name(
        conn: &PgConnection,
        subject_name: String,
    ) -> Result<Vec<Option<i32>>, ApiError> {
        use super::schema::schema_versions::dsl::{schema_versions, subject_id, version};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .filter(name.eq(&subject_name))
            .select(version)
            .order(version.asc())
            .load::<Option<i32>>(conn)
            .map_or_else(
                |_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                |versions| {
                    if versions.len() == 0 {
                        Err(ApiError::new(ApiErrorCode::SubjectNotFound))
                    } else {
                        Ok(versions)
                    }
                },
            )
    }

    pub fn latest_version_with_subject_name(
        conn: &PgConnection,
        subject_name: String,
    ) -> Result<Option<Option<i32>>, ApiError> {
        use super::schema::schema_versions::dsl::{schema_versions, subject_id, version};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .filter(name.eq(&subject_name))
            .select(version)
            .order(version.desc())
            .first::<Option<i32>>(conn)
            .optional()
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    pub fn get_schema_id_from_latest(
        conn: &PgConnection,
        subject_name: String,
    ) -> Result<(i64, i32, String), ApiError> {
        use super::schema::schema_versions::dsl::{
            schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{json, schemas};

        conn.transaction::<_, ApiError, _>(|| {
            let subject = Subject::get_by_name(conn, subject_name)?;

            let (schema_version, schema_id_result): (Option<i32>, i64) = match schema_versions
                .filter(subject_id.eq(subject.id))
                .order(version.desc())
                .select((version, schema_id))
                .first(conn)
            {
                Err(diesel::result::Error::NotFound) => {
                    Err(ApiError::new(ApiErrorCode::VersionNotFound))
                }
                Err(_) => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            let schema_json = match schemas.find(schema_id_result).select(json).first(conn) {
                Err(_) => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            Ok((
                schema_id_result,
                schema_version.ok_or_else(|| ApiError::new(ApiErrorCode::VersionNotFound))?,
                schema_json,
            ))
        })
    }

    /// TODO: This should return a struct, not a tuple as it's then hard to interface with
    /// this method
    pub fn get_schema_id(
        conn: &PgConnection,
        subject_name: String,
        schema_version: i32,
    ) -> Result<(i64, i32, String), ApiError> {
        use super::schema::schema_versions::dsl::{
            schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{json, schemas};

        conn.transaction::<_, ApiError, _>(|| {
            let subject = Subject::get_by_name(conn, subject_name)?;

            let schema_id_result = match schema_versions
                .filter(subject_id.eq(subject.id))
                .filter(version.eq(Some(schema_version)))
                .select(schema_id)
                .first(conn)
            {
                Err(diesel::result::Error::NotFound) => {
                    Err(ApiError::new(ApiErrorCode::VersionNotFound))
                }
                Err(_) => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            let schema_json = match schemas.find(schema_id_result).select(json).first(conn) {
                Err(_) => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            Ok((schema_id_result, schema_version as i32, schema_json))
        })
    }

    pub fn delete_subject_with_name(
        conn: &PgConnection,
        subject: String,
    ) -> Result<Vec<Option<i32>>, diesel::result::Error> {
        use super::schema::schema_versions::dsl::{
            id, schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        conn.transaction::<_, diesel::result::Error, _>(|| {
            Ok(schema_versions
                .inner_join(subjects.on(subject_id.eq(subjects_id)))
                .inner_join(schemas.on(schema_id.eq(schemas_id)))
                .filter(name.eq(&subject))
                .select((id, version, subject_id, schema_id))
                .load::<SchemaVersion>(conn)?
                .into_iter()
                .map(|entry| {
                    match entry.delete(&conn) {
                        Err(e) => {
                            info!("error deleting: {}", e);
                        }
                        _ => (),
                    };
                    entry.version
                })
                .collect())
        })
    }

    fn delete(&self, conn: &PgConnection) -> Result<(), diesel::result::Error> {
        use super::schema::configs::dsl::{configs, subject_id};
        use super::schema::schema_versions::dsl::{id, schema_versions};
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, subjects};

        conn.transaction::<_, diesel::result::Error, _>(|| {
            let schemas_delete = schemas.filter(schemas_id.eq(self.schema_id));
            let subjects_delete = subjects.filter(subjects_id.eq(self.subject_id));
            let schema_versions_delete = schema_versions.filter(id.eq(self.id));
            let configs_delete = configs.filter(subject_id.eq(self.subject_id));

            diesel::delete(schemas_delete).execute(conn)?;
            diesel::delete(subjects_delete).execute(conn)?;
            diesel::delete(schema_versions_delete).execute(conn)?;
            diesel::delete(configs_delete).execute(conn)?;

            Ok(())
        })
    }

    pub fn delete_version_with_subject(
        conn: &PgConnection,
        request: DeleteSchemaVersion,
    ) -> Result<i32, ApiError> {
        use super::Subject;

        use super::schema::schema_versions::dsl::version;

        let (subject, v) = (request.subject, request.version);

        conn.transaction::<_, ApiError, _>(|| {
            Subject::get_by_name(conn, subject.to_owned()).and_then(|subject| {
                diesel::delete(SchemaVersion::belonging_to(&subject).filter(version.eq(v)))
                    .execute(conn)
                    .or_else(|_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)))
                    .and_then(|o| match o {
                        0 => Err(ApiError::new(ApiErrorCode::VersionNotFound)),
                        _ => Ok(v),
                    })
            })
        })
    }
}

pub struct DeleteSchemaVersion {
    pub subject: String,
    pub version: i32,
}

impl Message for DeleteSchemaVersion {
    type Result = Result<i32, ApiError>;
}

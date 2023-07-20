use diesel::prelude::*;

use super::schema::*;
use super::schemas::Schema;
use super::subjects::Subject;

use crate::api::errors::{ApiAvroErrorCode, ApiError};

#[derive(Debug, Identifiable, Associations, Queryable)]
#[diesel(table_name = schema_versions)]
#[diesel(belongs_to(Schema))]
#[diesel(belongs_to(Subject))]
pub struct SchemaVersion {
    pub id: i64,
    pub version: Option<i32>,
    pub subject_id: i64,
    pub schema_id: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema_versions)]
pub struct NewSchemaVersion {
    pub version: Option<i32>,
    pub subject_id: i64,
    pub schema_id: i64,
}

pub type SchemaVersionFields = NewSchemaVersion;

impl SchemaVersion {
    pub fn insert(conn: &mut PgConnection, sv: NewSchemaVersion) -> Result<Self, ApiError> {
        use super::schema::schema_versions::dsl::schema_versions;
        diesel::insert_into(schema_versions)
            .values(&sv)
            .get_result::<Self>(conn)
            .map_err(|_| ApiError::new(ApiAvroErrorCode::BackendDatastoreError))
    }

    pub fn find(
        conn: &mut PgConnection,
        find_subject_id: i64,
        find_schema_id: i64,
    ) -> Result<Self, ApiError> {
        use super::schema::schema_versions::dsl::{schema_id, schema_versions, subject_id};

        schema_versions
            .filter(subject_id.eq(find_subject_id))
            .filter(schema_id.eq(find_schema_id))
            .get_result::<Self>(conn)
            .map_err(|_| ApiError::new(ApiAvroErrorCode::VersionNotFound))
    }

    pub fn with_schema_and_subject(
        conn: &mut PgConnection,
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
            .map_err(|_| ApiError::new(ApiAvroErrorCode::BackendDatastoreError))
    }

    pub fn versions_with_subject_name(
        conn: &mut PgConnection,
        subject_name: String,
    ) -> Result<Vec<Option<i32>>, ApiError> {
        use super::schema::schema_versions::dsl::{schema_versions, subject_id, version};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        match schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .filter(name.eq(&subject_name))
            .select(version)
            .order(version.asc())
            .load::<Option<i32>>(conn)
        {
            Err(_) => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
            Ok(versions) => {
                if versions.is_empty() {
                    Err(ApiError::new(ApiAvroErrorCode::SubjectNotFound))
                } else {
                    Ok(versions)
                }
            }
        }
    }

    pub fn latest_version_with_subject_name(
        conn: &mut PgConnection,
        subject_name: String,
    ) -> Result<Option<i32>, ApiError> {
        use super::schema::schema_versions::dsl::{schema_versions, subject_id, version};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        let res = schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .filter(name.eq(&subject_name))
            .select(version)
            .order(version.desc())
            .first::<Option<i32>>(conn);

        match res {
            Ok(v) => Ok(v),
            Err(diesel::NotFound) => Ok(None),
            _ => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
        }
    }

    pub fn get_schema_id_from_latest(
        conn: &mut PgConnection,
        subject_name: String,
    ) -> Result<(i64, i32, String), ApiError> {
        use super::schema::schema_versions::dsl::{
            schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{json, schemas};

        conn.transaction::<_, ApiError, _>(|conn| {
            let subject = Subject::get_by_name(conn, subject_name)?;

            let (schema_version, schema_id_result): (Option<i32>, i64) = match schema_versions
                .filter(subject_id.eq(subject.id))
                .order(version.desc())
                .select((version, schema_id))
                .first(conn)
            {
                Err(diesel::result::Error::NotFound) => {
                    Err(ApiError::new(ApiAvroErrorCode::VersionNotFound))
                }
                Err(_) => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            let schema_json = match schemas.find(schema_id_result).select(json).first(conn) {
                Err(_) => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            Ok((
                schema_id_result,
                schema_version.ok_or_else(|| ApiError::new(ApiAvroErrorCode::VersionNotFound))?,
                schema_json,
            ))
        })
    }

    /// TODO: This should return a struct, not a tuple as it's then hard to interface with
    /// this method
    pub fn get_schema_id(
        conn: &mut PgConnection,
        subject_name: String,
        schema_version: u32,
    ) -> Result<(i64, i32, String), ApiError> {
        use super::schema::schema_versions::dsl::{
            schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{json, schemas};

        conn.transaction::<_, ApiError, _>(|conn| {
            let subject = Subject::get_by_name(conn, subject_name)?;

            let schema_id_result = match schema_versions
                .filter(subject_id.eq(subject.id))
                .filter(version.eq(Some(schema_version as i32)))
                .select(schema_id)
                .first(conn)
            {
                Err(diesel::result::Error::NotFound) => {
                    Err(ApiError::new(ApiAvroErrorCode::VersionNotFound))
                }
                Err(_) => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            let schema_json = match schemas.find(schema_id_result).select(json).first(conn) {
                Err(_) => Err(ApiError::new(ApiAvroErrorCode::BackendDatastoreError)),
                Ok(o) => Ok(o),
            }?;

            Ok((schema_id_result, schema_version as i32, schema_json))
        })
    }

    pub fn delete_subject_with_name(
        conn: &mut PgConnection,
        subject: String,
    ) -> Result<Vec<Option<i32>>, diesel::result::Error> {
        use super::schema::schema_versions::dsl::{
            id, schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Ok(schema_versions
                .inner_join(subjects.on(subject_id.eq(subjects_id)))
                .inner_join(schemas.on(schema_id.eq(schemas_id)))
                .filter(name.eq(&subject))
                .select((id, version, subject_id, schema_id))
                .load::<Self>(conn)?
                .into_iter()
                .map(|entry| {
                    if let Err(e) = entry.delete(conn) {
                        info!("error deleting: {}", e);
                    }
                    entry.version
                })
                .collect())
        })
    }

    fn delete(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        use super::schema::configs::dsl::{configs, subject_id};
        use super::schema::schema_versions::dsl::{id, schema_versions};
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, subjects};

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
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
        conn: &mut PgConnection,
        request: DeleteSchemaVersion,
    ) -> Result<u32, ApiError> {
        use super::schema::schema_versions::dsl::version;

        let (subject, v) = (request.subject, request.version);

        conn.transaction::<_, ApiError, _>(|conn| {
            Subject::get_by_name(conn, subject.to_owned()).and_then(|subject| {
                diesel::delete(Self::belonging_to(&subject).filter(version.eq(v as i32)))
                    .execute(conn)
                    .map_err(|_| ApiError::new(ApiAvroErrorCode::BackendDatastoreError))
                    .and_then(|o| match o {
                        0 => Err(ApiError::new(ApiAvroErrorCode::VersionNotFound)),
                        _ => Ok(v),
                    })
            })
        })
    }
}

pub struct DeleteSchemaVersion {
    pub subject: String,
    pub version: u32,
}

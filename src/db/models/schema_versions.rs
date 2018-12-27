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

impl SchemaVersion {
    // TODO: I'm not happy with this. Positional arguments are so prone to errors it's not
    // even funny. Part of the issue is that we call this function usually after a select,
    // and said select only allows doing this with tuples. Sucks.
    pub fn from_tuple((id, version, subject_id, schema_id): (i64, Option<i32>, i64, i64)) -> Self {
        SchemaVersion {
            id: id,
            version: version,
            subject_id: subject_id,
            schema_id: schema_id,
        }
    }

    pub fn insert(
        conn: &PgConnection,
        schema_version: NewSchemaVersion,
    ) -> Result<SchemaVersion, ApiError> {
        use super::schema::schema_versions::dsl::*;
        diesel::insert_into(schema_versions)
            .values(&schema_version)
            .get_result::<SchemaVersion>(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    pub fn with_schema_and_subject(
        conn: &PgConnection,
        search_subject_id: i64,
        search_schema_id: i64,
    ) -> Result<usize, ApiError> {
        use super::schema::schema_versions::dsl::{id, schema_id, schema_versions, subject_id};
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, subjects};

        schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .inner_join(schemas.on(schema_id.eq(schemas_id)))
            .filter(subject_id.eq(search_subject_id))
            .filter(schema_id.eq(search_schema_id))
            .select(id)
            .execute(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    pub fn latest_version_with_subject_name(
        conn: &PgConnection,
        subject_name: String,
    ) -> Result<Option<i32>, ApiError> {
        use super::schema::schema_versions::dsl::{schema_versions, subject_id, version};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .filter(name.eq(&subject_name))
            .select(version)
            .order(version.desc())
            .first::<Option<i32>>(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    pub fn get_schema_id_from_latest(
        subject_name: String,
        conn: &PgConnection,
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

    pub fn get_schema_id(
        subject_name: String,
        schema_version: i32,
        conn: &PgConnection,
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
        subject: String,
        conn: &PgConnection,
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
                .load::<(i64, Option<i32>, i64, i64)>(conn)?
                .into_iter()
                .map(|entry| {
                    let sv = SchemaVersion::from_tuple(entry);
                    match sv.delete(&conn) {
                        Err(e) => {
                            info!("error deleting: {}", e);
                        }
                        _ => (),
                    };
                    sv.version
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

use actix::Message;
use avro_rs;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use crate::api::errors::{ApiError, ApiErrorCode};

use super::schema::*;

use super::{NewSchemaVersion, SchemaVersion, Subject};

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

#[derive(Debug, Insertable)]
#[table_name = "schemas"]
pub struct NewSchema {
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
    fn parse(data: String) -> Result<avro_rs::Schema, ApiError> {
        avro_rs::Schema::parse_str(&data)
            .map_err(|_| ApiError::new(ApiErrorCode::InvalidAvroSchema))
    }

    fn generate_fingerprint(data: String) -> Result<String, ApiError> {
        use sha2::Sha256;
        Ok(format!("{}", Schema::parse(data)?.fingerprint::<Sha256>()))
    }

    pub fn find_by_fingerprint(
        conn: &PgConnection,
        fingerprint: String,
    ) -> Result<Option<Schema>, ApiError> {
        use super::schema::schemas::dsl::{fingerprint2, schemas};
        Ok(schemas
            .filter(fingerprint2.eq(fingerprint))
            .load::<Schema>(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))?
            .pop())
    }

    pub fn register_new_version(
        conn: &PgConnection,
        registration: RegisterSchema,
    ) -> Result<Schema, ApiError> {
        let (subject, json) = (registration.subject, registration.schema);
        let fingerprint = Schema::generate_fingerprint(json.to_owned())?;

        conn.transaction::<_, ApiError, _>(|| {
            let db_schema = Schema::find_by_fingerprint(&conn, fingerprint.to_owned())?;
            match db_schema {
                Some(s) => {
                    let db_subject = Subject::get_by_name(conn, subject.to_owned())?;
                    match SchemaVersion::with_schema_and_subject(conn, db_subject.id, s.id) {
                        Ok(_) => Ok(s),
                        Err(_) => {
                            Schema::create_new_version(conn, None, fingerprint, subject, Some(s))
                        }
                    }
                }
                None => Schema::create_new_version(conn, Some(json), fingerprint, subject, None),
            }
        })
    }

    fn create_new_version(
        conn: &PgConnection,
        json: Option<String>,
        fingerprint: String,
        subject_name: String,
        db_schema: Option<Schema>,
    ) -> Result<Schema, ApiError> {
        let latest =
            SchemaVersion::latest_version_with_subject_name(conn, subject_name.to_owned())?;

        // If it already exists, we don't care, we just update and get the subject.
        let subject = Subject::insert(conn, subject_name)?;
        let (schema, new_version) = match latest {
            Some(latest_version) => {
                // TODO: Check compatibility first - implementation should be mostly in
                // https://github.com/flavray/avro-rs
                let sch = match json {
                    Some(j) => Schema::new(conn, j, fingerprint)?,
                    None => db_schema.ok_or(ApiError::new(ApiErrorCode::BackendDatastoreError))?,
                };
                (sch, latest_version + 1)
            }
            None => {
                // Create schema version for subject
                let sch = match json {
                    Some(j) => Schema::new(conn, j, fingerprint)?,
                    None => db_schema.ok_or(ApiError::new(ApiErrorCode::BackendDatastoreError))?,
                };
                (sch, 1)
            }
        };

        SchemaVersion::insert(
            conn,
            NewSchemaVersion {
                version: Some(new_version),
                subject_id: subject.id,
                schema_id: schema.id,
            },
        )?;
        // TODO: set compatibility
        Ok(schema)
    }

    pub fn new(conn: &PgConnection, json: String, fingerprint: String) -> Result<Schema, ApiError> {
        // TODO: we use the same in both fields. This means we don't do the same as
        // salsify
        let new_schema = NewSchema {
            json: json,
            fingerprint: fingerprint.to_owned(),
            fingerprint2: Some(fingerprint.to_owned()),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        Schema::insert(conn, new_schema)
    }

    pub fn insert(conn: &PgConnection, schema: NewSchema) -> Result<Schema, ApiError> {
        use super::schema::schemas::dsl::*;
        diesel::insert_into(schemas)
            .values(&schema)
            .get_result::<Schema>(conn)
            .map_err(|_| ApiError::new(ApiErrorCode::BackendDatastoreError))
    }

    // TODO: it's a bit crap to pre-optimise on this select. This function name smells too.
    pub fn get_json_by_id(conn: &PgConnection, schema_id: i64) -> Result<String, ApiError> {
        use super::schema::schemas::dsl::*;
        schemas
            .find(schema_id)
            .select(json)
            .get_result::<String>(conn)
            .or(Err(ApiError::new(ApiErrorCode::SchemaNotFound)))
    }
}

#[derive(Debug, Serialize)]
pub struct RegisterSchemaResponse {
    pub id: String,
}

pub struct RegisterSchema {
    pub subject: String,
    pub schema: String,
}

impl Message for RegisterSchema {
    type Result = Result<RegisterSchemaResponse, ApiError>;
}

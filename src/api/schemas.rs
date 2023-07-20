use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::api::errors::{ApiAvroErrorCode, ApiError};
use crate::db::models::{
    DeleteSchemaVersion, RegisterSchema, RegisterSchemaResponse, Schema, SchemaResponse,
    SchemaVersion,
};
use crate::db::{DbManage, DbPool};

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaBody {
    pub schema: String,
}

pub async fn get_schema(id: Path<i64>, db: Data<DbPool>) -> impl Responder {
    info!("method=get,id={}", id);

    let mut conn = db.connection()?;
    match Schema::get_by_id(&mut conn, id.into_inner()).map(|schema| SchemaResponse {
        schema: schema.json,
    }) {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e),
    }
}

pub async fn delete_schema_version(info: Path<(String, u32)>, db: Data<DbPool>) -> impl Responder {
    let q = info.into_inner();

    use crate::api::version::VersionLimit;

    let delete_schema_version = DeleteSchemaVersion {
        subject: q.0,
        version: q.1,
    };
    let mut conn = db.connection()?;
    if !delete_schema_version.version.within_limits() {
        return Err(ApiError::new(ApiAvroErrorCode::InvalidVersion));
    }
    match SchemaVersion::delete_version_with_subject(&mut conn, delete_schema_version) {
        Ok(r) => Ok(HttpResponse::Ok().body(format!("{}", r))),
        Err(e) => Err(e),
    }
}

pub async fn delete_schema_version_latest(
    subject: Path<String>,
    db: Data<DbPool>,
) -> impl Responder {
    let subject = subject.into_inner();

    use crate::api::version::VersionLimit;
    let mut conn = db.connection()?;

    let sv_response =
        crate::api::subjects::get_subject_version_from_db(&mut conn, subject.clone(), None)?;

    let delete_schema_version = DeleteSchemaVersion {
        subject,
        version: sv_response.version as u32,
    };
    if !delete_schema_version.version.within_limits() {
        return Err(ApiError::new(ApiAvroErrorCode::InvalidVersion));
    }
    match SchemaVersion::delete_version_with_subject(&mut conn, delete_schema_version) {
        Ok(r) => Ok(HttpResponse::Ok().body(format!("{}", r))),
        Err(e) => Err(e),
    }
}

pub async fn register_schema(
    subject: Path<String>,
    body: Json<SchemaBody>,
    db: Data<DbPool>,
) -> impl Responder {
    let mut conn = db.connection()?;
    let new_schema = RegisterSchema {
        subject: subject.to_owned(),
        schema: body.into_inner().schema,
    };
    match Schema::register_new_version(&mut conn, new_schema).map(|schema| RegisterSchemaResponse {
        id: format!("{}", schema.id),
    }) {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e),
    }
}

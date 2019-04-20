use actix_web::{
    web,
    web::{Data, Json, Path},
    HttpResponse,
};
use futures::Future;

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

pub fn get_schema(
    id: Path<i64>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    info!("method=get,id={}", id);

    web::block(move || {
        let conn = db.connection()?;
        Schema::get_by_id(&conn, id.into_inner()).map(|schema| SchemaResponse {
            schema: schema.json,
        })
    })
    .from_err()
    .then(|res| match res {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e),
    })
}

pub fn delete_schema_version(
    info: Path<(String, u32)>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let q = info.into_inner();

    web::block(move || {
        use crate::api::version::VersionLimit;

        let delete_schema_version = DeleteSchemaVersion {
            subject: q.0,
            version: q.1,
        };
        let conn = db.connection()?;
        if !delete_schema_version.version.within_limits() {
            return Err(ApiError::new(ApiAvroErrorCode::InvalidVersion));
        }
        SchemaVersion::delete_version_with_subject(&conn, delete_schema_version)
    })
    .from_err()
    .then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().body(format!("{}", r))),
        Err(e) => Err(e),
    })
}

pub fn register_schema(
    subject: Path<String>,
    body: Json<SchemaBody>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    web::block(move || {
        let conn = db.connection()?;
        let new_schema = RegisterSchema {
            subject: subject.to_owned(),
            schema: body.into_inner().schema,
        };
        Schema::register_new_version(&conn, new_schema).map(|schema| RegisterSchemaResponse {
            id: format!("{}", schema.id),
        })
    })
    .from_err()
    .then(|res| match res {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e),
    })
}

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use futures::Future;

use crate::api::errors::ApiError;
use crate::app::AppState;
use crate::db::models::{DeleteSchemaVersion, GetSchema, RegisterSchema};

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaBody {
    pub schema: String,
}

pub fn get_schema(
    id: Path<i64>,
    data: Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    info!("method=get,id={}", id);

    data.db
        .send(GetSchema {
            id: id.into_inner(),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(response) => Ok(HttpResponse::Ok().json(response)),
            Err(e) => Err(e),
        })
}

pub fn delete_schema_version(
    info: Path<(String, u32)>,
    data: Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let q = info.into_inner();

    data.db
        .send(DeleteSchemaVersion {
            subject: q.0,
            version: q.1,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().body(format!("{}", r))),
            Err(e) => Err(e),
        })
}

pub fn register_schema(
    subject: Path<String>,
    body: Json<SchemaBody>,
    data: Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    data.db
        .send(RegisterSchema {
            subject: subject.to_owned(),
            schema: body.into_inner().schema,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(response) => Ok(HttpResponse::Ok().json(response)),
            Err(e) => Err(e),
        })
}

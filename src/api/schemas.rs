use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, Path, State};
use futures::future::{result, Future};

use crate::api::errors::{ApiError, ApiErrorCode};
use crate::app::{AppState, VersionLimit};
use crate::db::models::{DeleteSchemaVersion, GetSchema, RegisterSchema};

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaBody {
    pub schema: String,
}

pub fn get_schema(id: Path<i64>, state: State<AppState>) -> FutureResponse<HttpResponse> {
    info!("method=get,id={}", id);

    state
        .db
        .send(GetSchema {
            id: id.into_inner(),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(schema) => Ok(HttpResponse::Ok().json(schema)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn delete_schema_version(
    info: Path<(String, u32)>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    let q = info.into_inner();

    match q.1.within_limits() {
        false => {
            return result(Ok(
                ApiError::new(ApiErrorCode::InvalidVersion).http_response()
            ))
            .responder();
        }
        _ => (),
    }

    state
        .db
        .send(DeleteSchemaVersion {
            subject: q.0,
            version: q.1 as i32,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().body(format!("{}", r))),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn register_schema(
    subject: Path<String>,
    body: Json<SchemaBody>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(RegisterSchema {
            subject: subject.to_owned(),
            schema: body.into_inner().schema,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(response) => Ok(HttpResponse::Ok().json(response)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

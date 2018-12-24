use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Path, State};
use futures::future::Future;

use crate::db::models::{DeleteSchemaVersion, GetSchema};
use crate::AppState;

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

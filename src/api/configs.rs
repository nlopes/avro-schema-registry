use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, Path, State};
use futures::future::Future;

use crate::db::models::{GetConfig, GetSubjectConfig, SetConfig, SetSubjectConfig};
use crate::AppState;

pub fn get_config(state: State<AppState>) -> FutureResponse<HttpResponse> {
    info!("path=/config,method=get");

    state
        .db
        .send(GetConfig {})
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn put_config(
    (body, state): (Json<SetConfig>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    let compatibility = body.compatibility;
    info!("method=put,compatibility={}", compatibility);

    state
        .db
        .send(SetConfig {
            compatibility: compatibility,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn get_subject_config(
    (subject_path, state): (Path<String>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    let subject = subject_path.into_inner();
    info!("method=get,subject={}", subject);

    state
        .db
        .send(GetSubjectConfig { subject: subject })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn put_subject_config(
    (subject_path, body, state): (Path<String>, Json<SetConfig>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    let subject = subject_path.into_inner();
    let compatibility = body.compatibility;
    info!(
        "method=put,subject={},compatibility={}",
        subject, compatibility
    );

    state
        .db
        .send(SetSubjectConfig {
            subject: subject,
            compatibility: compatibility,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

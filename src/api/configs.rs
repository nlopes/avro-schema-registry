use actix_web::{
    web::{Data, Json, Path},
    Error, HttpResponse,
};

use futures::Future;

use crate::app::AppState;
use crate::db::models::{GetConfig, GetSubjectConfig, SetConfig, SetSubjectConfig};

pub fn get_config(data: Data<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    info!("path=/config,method=get");

    data.db
        .send(GetConfig {})
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
}

pub fn put_config(
    body: Json<SetConfig>,
    data: Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let compatibility = body.compatibility;
    info!("method=put,compatibility={}", compatibility);

    data.db
        .send(SetConfig {
            compatibility: compatibility,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
}

/// Get compatibility level for a subject.
pub fn get_subject_config(
    subject_path: Path<String>,
    data: Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let subject = subject_path.into_inner();
    info!("method=get,subject={}", subject);

    data.db
        .send(GetSubjectConfig { subject: subject })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
}

/// Update compatibility level for the specified subject.
///
/// *Note:* The confluent schema registry does not return "Subject not found" if the
/// subject does not exist, due to the way they map configs to subjects. We map them
/// internally to subject_id's therefore, we can *and will* return "Schema not found" if
/// no subject is found with the given name.
pub fn put_subject_config(
    subject_path: Path<String>,
    body: Json<SetConfig>,
    data: Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let subject = subject_path.into_inner();
    let compatibility = body.compatibility;
    info!(
        "method=put,subject={},compatibility={}",
        subject, compatibility
    );

    data.db
        .send(SetSubjectConfig {
            subject: subject,
            compatibility: compatibility,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Ok(e.http_response()),
        })
}

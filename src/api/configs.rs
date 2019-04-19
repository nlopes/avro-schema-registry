use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use futures::Future;

use crate::api::errors::ApiError;
use crate::db::models::{GetConfig, GetSubjectConfig, SetConfig, SetSubjectConfig};
use crate::db::PoolerAddr;

pub fn get_config(db: Data<PoolerAddr>) -> impl Future<Item = HttpResponse, Error = ApiError> {
    info!("path=/config,method=get");

    db.send(GetConfig {}).from_err().and_then(|res| match res {
        Ok(config) => Ok(HttpResponse::Ok().json(config)),
        Err(e) => Err(e),
    })
}

pub fn put_config(
    body: Json<SetConfig>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let compatibility = body.compatibility;
    info!("method=put,compatibility={}", compatibility);

    db.send(SetConfig { compatibility })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Err(e),
        })
}

/// Get compatibility level for a subject.
pub fn get_subject_config(
    subject_path: Path<String>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let subject = subject_path.into_inner();
    info!("method=get,subject={}", subject);

    db.send(GetSubjectConfig { subject })
        .from_err()
        .and_then(|res| match res {
            Ok(config) => Ok(HttpResponse::Ok().json(config)),
            Err(e) => Err(e),
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
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let subject = subject_path.into_inner();
    let compatibility = body.compatibility;
    info!(
        "method=put,subject={},compatibility={}",
        subject, compatibility
    );

    db.send(SetSubjectConfig {
        subject,
        compatibility,
    })
    .from_err()
    .and_then(|res| match res {
        Ok(config) => Ok(HttpResponse::Ok().json(config)),
        Err(e) => Err(e),
    })
}

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use futures::Future;

use crate::api::{errors::ApiError, SchemaBody};
use crate::db::models::{
    DeleteSubject, GetSubjectVersion, GetSubjectVersions, GetSubjects, SchemaResponse,
    VerifySchemaRegistration,
};
use crate::db::PoolerAddr;

pub fn get_subjects(db: Data<PoolerAddr>) -> impl Future<Item = HttpResponse, Error = ApiError> {
    db.send(GetSubjects {})
        .from_err()
        .and_then(|res| match res {
            Ok(subjects) => Ok(HttpResponse::Ok().json(subjects.content)),
            Err(e) => Err(e),
        })
}

pub fn get_subject_versions(
    subject: Path<String>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    db.send(GetSubjectVersions {
        subject: subject.into_inner(),
    })
    .from_err()
    .and_then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
        Err(e) => Err(e),
    })
}

pub fn delete_subject(
    subject: Path<String>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    db.send(DeleteSubject {
        subject: subject.into_inner(),
    })
    .from_err()
    .and_then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
        Err(e) => Err(e),
    })
}

// TODO(nlopes): maybe new type here
//
// According to
// https://docs.confluent.io/3.1.0/schema-registry/docs/api.html#get--subjects-(string-%20subject)-versions-(versionId-%20version)
// the Version ID should be in the range of 1 to 2^31-1, which isn't u32. We should create
// a new type with the boundaries of this.
pub fn get_subject_version(
    info: Path<(String, u32)>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let q = info.into_inner();

    db.send(GetSubjectVersion {
        subject: q.0,
        version: Some(q.1),
    })
    .from_err()
    .and_then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r)),
        Err(e) => Err(e),
    })
}

pub fn get_subject_version_latest(
    subject: Path<String>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    db.send(GetSubjectVersion {
        subject: subject.into_inner(),
        version: None,
    })
    .from_err()
    .and_then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r)),
        Err(e) => Err(e),
    })
}

// TODO: for now, do the same as for `get_subject_version` and then extract only the
// schema
pub fn get_subject_version_schema(
    info: Path<(String, u32)>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let q = info.into_inner();

    db.send(GetSubjectVersion {
        subject: q.0,
        version: Some(q.1),
    })
    .from_err()
    .and_then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
        Err(e) => Err(e),
    })
}

pub fn get_subject_version_latest_schema(
    subject: Path<String>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    db.send(GetSubjectVersion {
        subject: subject.into_inner(),
        version: None,
    })
    .from_err()
    .and_then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
        Err(e) => Err(e),
    })
}

pub fn post_subject(
    subject: Path<String>,
    body: Json<SchemaBody>,
    db: Data<PoolerAddr>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    db.send(VerifySchemaRegistration {
        subject: subject.into_inner(),
        schema: body.into_inner().schema,
    })
    .from_err()
    .and_then(|res| match res {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e),
    })
}

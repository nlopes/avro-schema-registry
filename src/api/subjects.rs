use std::collections::HashMap;

use crate::db::models::{DeleteSubject, GetSubjectVersions, GetSubjects, RegisterSchema};
use crate::AppState;

use actix_web::{
    AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json, Path, Query, State,
};
use futures::future::Future;

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaBody {
    schema: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectInfo {
    subject: String,
}

pub fn post_subject(
    path: Path<SubjectInfo>,
    _query: Query<HashMap<String, String>>,
    body: Json<SchemaBody>,
) -> HttpResponse {
    println!("GOT HERE!");
    unimplemented!();
}

pub fn delete_subject(
    subject: Path<String>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(DeleteSubject {
            subject: subject.into_inner(),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

// TODO(nlopes): maybe new type here
//
// According to
// https://docs.confluent.io/3.1.0/schema-registry/docs/api.html#get--subjects-(string-%20subject)-versions-(versionId-%20version)
// the Version ID should be in the range of 1 to 2^31-1, which isn't u32. We should create
// a new type with the boundaries of this.
pub fn get_subject_version(info: Path<(String, u32)>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("Subject: {}\nVersion: {}", info.0, info.1))
}

pub fn get_subject_version_latest(subject: Path<String>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("Subject: {}\nVersion: latest YEEES", subject))
}

pub fn get_subject_versions(
    subject: Path<String>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(GetSubjectVersions {
            subject: subject.into_inner(),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn get_subjects(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(GetSubjects {})
        .from_err()
        .and_then(|res| match res {
            Ok(subjects) => Ok(HttpResponse::Ok().json(subjects.content)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn register_schema(
    path: Path<SubjectInfo>,
    _query: Query<HashMap<String, String>>,
    body: Json<SchemaBody>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(RegisterSchema {
            subject: path.subject.to_owned(),
            schema: body.into_inner().schema,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(response) => Ok(HttpResponse::Ok().json(response)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

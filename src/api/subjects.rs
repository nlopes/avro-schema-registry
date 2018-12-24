use crate::db::models::{
    DeleteSubject, GetSubjectVersion, GetSubjectVersions, GetSubjects, RegisterSchema,
    SchemaResponse, VerifySchemaRegistration,
};
use crate::AppState;

use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json, Path, State};
use futures::future::Future;

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaBody {
    schema: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectInfo {
    subject: String,
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
pub fn get_subject_version(
    info: Path<(String, u32)>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    let q = info.into_inner();

    state
        .db
        .send(GetSubjectVersion {
            subject: q.0,
            version: Some(q.1),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(r)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn get_subject_version_latest(
    subject: Path<String>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(GetSubjectVersion {
            subject: subject.into_inner(),
            version: None,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(r)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

// TODO: for now, do the same as for `get_subject_version` and then extract only the
// schema
pub fn get_subject_version_schema(
    info: Path<(String, u32)>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    let q = info.into_inner();

    state
        .db
        .send(GetSubjectVersion {
            subject: q.0,
            version: Some(q.1),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn get_subject_version_latest_schema(
    subject: Path<String>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(GetSubjectVersion {
            subject: subject.into_inner(),
            version: None,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

pub fn register_schema(
    path: Path<SubjectInfo>,
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

pub fn post_subject(
    subject: Path<String>,
    body: Json<SchemaBody>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(VerifySchemaRegistration {
            subject: subject.into_inner(),
            schema: body.into_inner().schema,
        })
        .from_err()
        .and_then(|res| match res {
            Ok(response) => Ok(HttpResponse::Ok().json(response)),
            Err(e) => Ok(e.http_response()),
        })
        .responder()
}

use actix_web::{
    web,
    web::{Data, Json, Path},
    HttpResponse,
};
use futures::Future;

use crate::api::{
    errors::{ApiAvroErrorCode, ApiError},
    SchemaBody,
};
use crate::db::models::{
    DeleteSubjectResponse, GetSubjectVersionResponse, Schema, SchemaResponse, SchemaVersion,
    Subject, SubjectList, SubjectVersionsResponse,
};
use crate::db::{DbManage, DbPool};

pub fn get_subjects(db: Data<DbPool>) -> impl Future<Item = HttpResponse, Error = ApiError> {
    web::block(move || {
        let conn = db.connection()?;
        Subject::distinct_names(&conn).map(|content| SubjectList { content })
    })
    .from_err()
    .then(|res| match res {
        Ok(subjects) => Ok(HttpResponse::Ok().json(subjects.content)),
        Err(e) => Err(e),
    })
}

pub fn get_subject_versions(
    subject: Path<String>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    //subject.into_inner()
    web::block(move || {
        let conn = db.connection()?;
        SchemaVersion::versions_with_subject_name(&conn, subject.into_inner())
            .map(|versions| SubjectVersionsResponse { versions })
    })
    .from_err()
    .then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
        Err(e) => Err(e),
    })
}

pub fn delete_subject(
    subject: Path<String>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    web::block(move || {
        let conn = db.connection()?;
        Subject::delete_by_name(&conn, subject.into_inner())
            .map(|versions| DeleteSubjectResponse { versions })
    })
    .from_err()
    .then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
        Err(e) => Err(e),
    })
}

fn get_subject_version_from_db(
    conn: &diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    subject: String,
    version: Option<u32>,
) -> Result<GetSubjectVersionResponse, ApiError> {
    use crate::api::version::VersionLimit;

    match version {
        Some(v) => {
            if !v.within_limits() {
                return Err(ApiError::new(ApiAvroErrorCode::InvalidVersion));
            }
            SchemaVersion::get_schema_id(&conn, subject.to_string(), v)
        }
        None => SchemaVersion::get_schema_id_from_latest(&conn, subject.to_string()),
    }
    .map(|o| GetSubjectVersionResponse {
        subject: subject.to_string(),
        id: o.0,
        version: o.1,
        schema: o.2,
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
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let q = info.into_inner();

    web::block(move || {
        let conn = db.connection()?;
        get_subject_version_from_db(&conn, q.0, Some(q.1))
    })
    .from_err()
    .then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r)),
        Err(e) => Err(e),
    })
}

pub fn get_subject_version_latest(
    subject: Path<String>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    web::block(move || {
        let conn = db.connection()?;
        get_subject_version_from_db(&conn, subject.into_inner(), None)
    })
    .from_err()
    .then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(r)),
        Err(e) => Err(e),
    })
}

// TODO: for now, do the same as for `get_subject_version` and then extract only the
// schema
pub fn get_subject_version_schema(
    info: Path<(String, u32)>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let q = info.into_inner();

    web::block(move || {
        let conn = db.connection()?;
        get_subject_version_from_db(&conn, q.0, Some(q.1))
    })
    .from_err()
    .then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
        Err(e) => Err(e),
    })
}

pub fn get_subject_version_latest_schema(
    subject: Path<String>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    web::block(move || {
        let conn = db.connection()?;
        get_subject_version_from_db(&conn, subject.into_inner(), None)
    })
    .from_err()
    .then(|res| match res {
        Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
        Err(e) => Err(e),
    })
}

pub fn post_subject(
    subject: Path<String>,
    body: Json<SchemaBody>,
    db: Data<DbPool>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    web::block(move || {
        let conn = db.connection()?;
        Schema::verify_registration(&conn, subject.into_inner(), body.into_inner().schema)
    })
    .from_err()
    .then(|res| match res {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e),
    })
}

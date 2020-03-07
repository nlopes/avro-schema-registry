use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::api::{
    errors::{ApiAvroErrorCode, ApiError},
    SchemaBody,
};
use crate::db::models::{
    DeleteSubjectResponse, GetSubjectVersionResponse, Schema, SchemaResponse, SchemaVersion,
    Subject, SubjectList, SubjectVersionsResponse,
};
use crate::db::{DbManage, DbPool};

pub async fn get_subjects(db: Data<DbPool>) -> impl Responder {
    let conn = db.connection()?;
    match Subject::distinct_names(&conn).map(|content| SubjectList { content }) {
        Ok(subjects) => Ok(HttpResponse::Ok().json(subjects.content)),
        Err(e) => Err(e),
    }
}

pub async fn get_subject_versions(subject: Path<String>, db: Data<DbPool>) -> impl Responder {
    //subject.into_inner()
    let conn = db.connection()?;
    match SchemaVersion::versions_with_subject_name(&conn, subject.into_inner())
        .map(|versions| SubjectVersionsResponse { versions })
    {
        Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
        Err(e) => Err(e),
    }
}

pub async fn delete_subject(subject: Path<String>, db: Data<DbPool>) -> impl Responder {
    let conn = db.connection()?;
    match Subject::delete_by_name(&conn, subject.into_inner())
        .map(|versions| DeleteSubjectResponse { versions })
    {
        Ok(r) => Ok(HttpResponse::Ok().json(r.versions)),
        Err(e) => Err(e),
    }
}

/// `get_subject_version_from_db` fetches a specific subject version pair from the
/// database, given a subject name and an optional version. If the version is not given,
/// then we get the latest schema id.
pub(crate) fn get_subject_version_from_db(
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
pub async fn get_subject_version(info: Path<(String, u32)>, db: Data<DbPool>) -> impl Responder {
    let q = info.into_inner();

    let conn = db.connection()?;
    match get_subject_version_from_db(&conn, q.0, Some(q.1)) {
        Ok(r) => Ok(HttpResponse::Ok().json(r)),
        Err(e) => Err(e),
    }
}

pub async fn get_subject_version_latest(subject: Path<String>, db: Data<DbPool>) -> impl Responder {
    let conn = db.connection()?;
    match get_subject_version_from_db(&conn, subject.into_inner(), None) {
        Ok(r) => Ok(HttpResponse::Ok().json(r)),
        Err(e) => Err(e),
    }
}

// TODO: for now, do the same as for `get_subject_version` and then extract only the
// schema
pub async fn get_subject_version_schema(
    info: Path<(String, u32)>,
    db: Data<DbPool>,
) -> impl Responder {
    let q = info.into_inner();

    let conn = db.connection()?;
    match get_subject_version_from_db(&conn, q.0, Some(q.1)) {
        Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
        Err(e) => Err(e),
    }
}

pub async fn get_subject_version_latest_schema(
    subject: Path<String>,
    db: Data<DbPool>,
) -> impl Responder {
    let conn = db.connection()?;
    match get_subject_version_from_db(&conn, subject.into_inner(), None) {
        Ok(r) => Ok(HttpResponse::Ok().json(SchemaResponse { schema: r.schema })),
        Err(e) => Err(e),
    }
}

pub async fn post_subject(
    subject: Path<String>,
    body: Json<SchemaBody>,
    db: Data<DbPool>,
) -> impl Responder {
    let conn = db.connection()?;
    match Schema::verify_registration(&conn, subject.into_inner(), body.into_inner().schema) {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(e),
    }
}

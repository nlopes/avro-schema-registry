use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use crate::db::models::{Config, ConfigCompatibility, SetConfig};
use crate::db::{DbManage, DbPool};

pub async fn get_config(db: Data<DbPool>) -> impl Responder {
    info!("path=/config,method=get");

    let conn = db.connection()?;
    match Config::get_global_compatibility(&conn).and_then(ConfigCompatibility::new) {
        Ok(config) => Ok(HttpResponse::Ok().json(config)),
        Err(e) => Err(e),
    }
}

pub async fn put_config(body: Json<SetConfig>, db: Data<DbPool>) -> impl Responder {
    let compatibility = body.compatibility;
    info!("method=put,compatibility={}", compatibility);

    let conn = db.connection()?;
    match Config::set_global_compatibility(&conn, &compatibility.valid()?.to_string())
        .and_then(ConfigCompatibility::new)
    {
        Ok(config) => Ok(HttpResponse::Ok().json(config)),
        Err(e) => Err(e),
    }
}

/// Get compatibility level for a subject.
pub async fn get_subject_config(subject_path: Path<String>, db: Data<DbPool>) -> impl Responder {
    let subject = subject_path.into_inner();
    info!("method=get,subject={}", subject);

    let conn = db.connection()?;
    match Config::get_with_subject_name(&conn, subject).and_then(ConfigCompatibility::new) {
        Ok(config) => Ok(HttpResponse::Ok().json(config)),
        Err(e) => Err(e),
    }
}

/// Update compatibility level for the specified subject.
///
/// *Note:* The confluent schema registry does not return "Subject not found" if the
/// subject does not exist, due to the way they map configs to subjects. We map them
/// internally to subject_id's therefore, we can *and will* return "Schema not found" if
/// no subject is found with the given name.
pub async fn put_subject_config(
    subject_path: Path<String>,
    body: Json<SetConfig>,
    db: Data<DbPool>,
) -> impl Responder {
    let subject = subject_path.into_inner();
    let compatibility = body.compatibility;
    info!(
        "method=put,subject={},compatibility={}",
        subject, compatibility
    );

    let conn = db.connection()?;
    match Config::set_with_subject_name(&conn, subject, compatibility.valid()?.to_string())
        .and_then(ConfigCompatibility::new)
    {
        Ok(config) => Ok(HttpResponse::Ok().json(config)),
        Err(e) => Err(e),
    }
}

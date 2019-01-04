use std::env;

use actix_web::{actix::Addr, http::Method, middleware::Logger, App};

use crate::api;
use crate::db::ConnectionPooler;
use crate::health;
use crate::middleware;

pub struct AppState {
    pub db: Addr<ConnectionPooler>,
}

pub trait VersionLimit {
    fn within_limits(&self) -> bool;
}

impl VersionLimit for u32 {
    fn within_limits(&self) -> bool {
        *self > 0 && *self < 2_147_483_648
    }
}

pub fn create_monitoring_app() -> App {
    App::new()
        .prefix("_")
        .middleware(Logger::default())
        .resource("/health_check", |r| r.method(Method::GET).h(health::status))
        .resource("/metrics", |r| r.method(Method::GET).h(health::metrics))
}

pub fn create_avro_api_app() -> App<AppState> {
    // TODO: remove this magic number 4?
    let db_addr = ConnectionPooler::init(4);
    create_avro_api_app_with_state(AppState {
        db: db_addr.clone(),
    })
}

pub fn create_avro_api_app_with_state(state: AppState) -> App<AppState> {
    let password =
        env::var("SCHEMA_REGISTRY_PASSWORD").expect("Must pass a schema registry password");

    App::with_state(state)
        .middleware(Logger::default())
        .middleware(middleware::VerifyAcceptHeader)
        .middleware(middleware::VerifyAuthorization::new(&password))
        .resource("/config", |r| {
            r.get().with(api::get_config);
            r.put().with(api::put_config)
        })
        .resource("/config/{subject}", |r| {
            r.get().with(api::get_subject_config);
            r.put().with(api::put_subject_config)
        })
        .resource("/subjects", |r| r.get().with(api::get_subjects))
        .resource("/subjects/{subject}", |r| {
            r.post().with(api::post_subject);
            r.delete().with(api::delete_subject)
        })
        .resource("/subjects/{subject}/versions", |r| {
            r.get().with(api::get_subject_versions);
            r.post().with(api::register_schema)
        })
        .resource("/subjects/{subject}/versions/latest", |r| {
            r.get().with(api::get_subject_version_latest)
            // TODO: r.delete().with(api::delete_schema_version_latest)
        })
        .resource("/subjects/{subject}/versions/{version}", |r| {
            r.get().with(api::get_subject_version);
            r.delete().with(api::delete_schema_version)
        })
        .resource("/subjects/{subject}/versions/latest/schema", |r| {
            r.get().with(api::get_subject_version_latest_schema)
        })
        .resource("/subjects/{subject}/versions/{version}/schema", |r| {
            r.get().with(api::get_subject_version_schema)
        })
        .resource("/schemas/ids/{id}", |r| r.get().with(api::get_schema))
}

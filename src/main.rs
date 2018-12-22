#![feature(custom_attribute)]
#![feature(result_map_or_else)]
#![feature(type_ascription)]

use std::env;
use std::error::Error;

extern crate actix;
extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate json;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use actix_web::{actix::Addr, http::Method, middleware::Logger, server::HttpServer, App, Result};

mod api;
mod db;
mod health;
mod middleware;

pub struct AppState {
    db: Addr<db::ConnectionPooler>,
}

fn main() -> Result<(), Box<Error>> {
    env::set_var("RUST_LOG", "actix_web=debug,avro_schema_registry=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let sys = actix_web::actix::System::new("avroapi");
    let password = env::var("SCHEMA_REGISTRY_PASSWORD")?;

    let db_addr = db::ConnectionPooler::init();

    // TODO: why do I need to use move? I don't understand
    HttpServer::new(move || {
        vec![
            App::new()
                .prefix("_")
                .middleware(Logger::default())
                .route("/health_check", Method::GET, health::status)
                .route("/metrics", Method::GET, health::metrics)
                .boxed(),
            App::with_state(AppState {
                db: db_addr.clone(),
            })
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
            })
            .resource("/subjects/{subject}/versions/{version}", |r| {
                r.get().with(api::get_subject_version)
            })
            .resource("/schemas/ids/{id}", |r| r.get().with(api::get_schema))
            .boxed(),
        ]
    })
    .bind(format!("127.0.0.1:{}", env::var("PORT")?))
    .unwrap()
    .shutdown_timeout(2)
    .start();

    sys.run();
    Ok(())
}

use std::env;

use actix::Addr;
use actix_web::web;

use crate::api;
use crate::db::ConnectionPooler;
use crate::health;
use crate::middleware;

pub struct AppState {
    pub db: Addr<ConnectionPooler>,
}

pub fn create_api_state() -> AppState {
    // TODO: remove this magic number 4?
    let db_addr = ConnectionPooler::init(4);
    AppState {
        db: db_addr.clone(),
    }
}

pub fn monitoring_routing(cfg: &mut web::RouterConfig) {
    cfg.service(
        web::scope("_")
            .service(web::resource("/health_check").to(health::status))
            .service(web::resource("/metrics").to(health::metrics)),
    );
}

pub fn api_routing(cfg: &mut web::RouterConfig) {
    let password =
        env::var("SCHEMA_REGISTRY_PASSWORD").expect("Must pass a schema registry password");

    cfg.service(
        web::scope("")
            .wrap(middleware::VerifyAcceptHeader)
            .wrap(middleware::VerifyAuthorization::new(&password))
            .service(
                web::resource("/config")
                    .route(web::get().to_async(api::get_config))
                    .route(web::put().to_async(api::put_config)),
            )
            .service(
                web::resource("/config/{subject}")
                    .route(web::get().to_async(api::get_subject_config))
                    .route(web::put().to_async(api::put_subject_config)),
            )
            .service(
                web::scope("/subjects")
                    .service(web::resource("").to_async(api::get_subjects))
                    .service(
                        web::resource("/{subject}")
                            .route(web::post().to_async(api::post_subject))
                            .route(web::delete().to_async(api::delete_subject)),
                    )
                    .service(
                        web::resource("/{subject}/versions")
                            .route(web::get().to_async(api::get_subject_versions))
                            .route(web::post().to_async(api::register_schema)),
                    )
                    .service(
                        web::resource("/{subject}/versions/latest")
                            .route(web::get().to_async(api::get_subject_version_latest)),
                        // TODO: .route(web::delete().to_async(api::delete_schema_version_latest)),
                    )
                    .service(
                        web::resource("/{subject}/versions/{version}")
                            .route(web::get().to_async(api::get_subject_version))
                            .route(web::delete().to_async(api::delete_schema_version)),
                    )
                    .service(
                        web::resource("/{subject}/versions/latest/schema")
                            .to_async(api::get_subject_version_latest_schema),
                    )
                    .service(
                        web::resource("/{subject}/versions/{version}/schema")
                            .to_async(api::get_subject_version_schema),
                    ),
            )
            .service(web::resource("/schemas/ids/{id}").to_async(api::get_schema)),
    );
}

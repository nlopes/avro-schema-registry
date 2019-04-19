use std::env;

use actix_web::web;

use crate::api;
use crate::health;
use crate::middleware;

pub fn monitoring_routing(cfg: &mut web::RouterConfig) {
    cfg.service(
        web::scope("_")
            .service(web::resource("/health_check").route(web::get().to(health::status)))
            .service(web::resource("/metrics").route(web::get().to(health::metrics))),
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
                web::resource("/compatibility/subjects/{subject}/versions/{version}")
                    .route(web::post().to_async(api::check_compatibility)),
            )
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
            .service(web::resource("/schemas/ids/{id}").route(web::get().to_async(api::get_schema)))
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
            ),
    );
}

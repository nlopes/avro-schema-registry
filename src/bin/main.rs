use std::env;

use actix_web::{
    middleware::{Compat, Logger},
    web::Data,
    App, HttpServer,
};
use actix_web_prom::PrometheusMetricsBuilder;
use sentry::integrations::panic as sentry_panic;
use sentry::IntoDsn;

use avro_schema_registry::app;
use avro_schema_registry::db::{DbManage, DbPool};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,avro_schema_registry=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let _sentry_client = sentry::init(sentry::ClientOptions {
        dsn: env::var("SENTRY_URL").ok().into_dsn().unwrap(),
        release: Some(std::borrow::Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        ..Default::default()
    });

    let _integration = sentry_panic::PanicIntegration::default().add_extractor(|_info| None);
    let prometheus = PrometheusMetricsBuilder::new("avro_schema_registry")
        .endpoint("/_/metrics")
        .build()
        .unwrap();

    HttpServer::new(move || {
        let db_pool = DbPool::new_pool(None);

        App::new()
            .wrap(Compat::new(Logger::default()))
            .wrap(prometheus.clone())
            .configure(app::monitoring_routing)
            .app_data(Data::new(db_pool))
            .configure(app::api_routing)
    })
    .bind(env::var("DEFAULT_HOST").unwrap_or_else(|_| "127.0.0.1:8080".to_string()))?
    .shutdown_timeout(2)
    .run()
    .await
}

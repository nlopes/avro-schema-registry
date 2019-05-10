use std::env;
use std::error::Error;

use actix_rt;
use actix_web::{middleware::Logger, App, HttpServer, Result};
use actix_web_prom::PrometheusMetrics;
use sentry;
use sentry::integrations::panic::register_panic_handler;
use sentry::internals::IntoDsn;

use avro_schema_registry::app;
use avro_schema_registry::db::{DbManage, DbPool};

fn main() -> Result<(), Box<Error>> {
    env::set_var("RUST_LOG", "actix_web=debug,avro_schema_registry=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let sys = actix_rt::System::new("avroapi");

    let _sentry_client = sentry::init(sentry::ClientOptions {
        dsn: env::var("SENTRY_URL").ok().into_dsn().unwrap(),
        release: Some(std::borrow::Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
        ..Default::default()
    });
    register_panic_handler();

    let prometheus = PrometheusMetrics::new("avro_schema_registry", "/_/metrics");

    HttpServer::new(move || {
        let db_pool = DbPool::new_pool(None);

        App::new()
            .wrap(Logger::default())
            .wrap(prometheus.clone())
            .configure(app::monitoring_routing)
            .data(db_pool)
            .configure(app::api_routing)
    })
    .bind(env::var("DEFAULT_HOST").unwrap_or_else(|_| "127.0.0.1:8080".to_string()))
    .unwrap()
    .shutdown_timeout(2)
    .start();

    sys.run()?;
    Ok(())
}

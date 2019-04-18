#![feature(custom_attribute)]
#![feature(result_map_or_else)]
#![feature(type_ascription)]

use std::env;
use std::error::Error;

use actix_rt;
use actix_web::{middleware::Logger, App, HttpServer, Result};
extern crate sentry;

use avro_schema_registry::app;

fn main() -> Result<(), Box<Error>> {
    let _guard = sentry::init(env::var("SENTRY_URL").expect("env must have SENTRY_URL"));

    env::set_var("RUST_LOG", "actix_web=debug,avro_schema_registry=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let sys = actix_rt::System::new("avroapi");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(app::monitoring_routing)
            .data(app::create_api_state())
            .configure(app::api_routing)
    })
    .bind(format!("127.0.0.1:{}", env::var("PORT")?))
    .unwrap()
    .shutdown_timeout(2)
    .start();

    sys.run()?;
    Ok(())
}

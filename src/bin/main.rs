#![feature(custom_attribute)]
#![feature(result_map_or_else)]
#![feature(type_ascription)]

use std::env;
use std::error::Error;

use actix_web::{server::HttpServer, Result};

use avro_schema_registry::app;

fn main() -> Result<(), Box<Error>> {
    env::set_var("RUST_LOG", "actix_web=debug,avro_schema_registry=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let sys = actix_web::actix::System::new("avroapi");

    // TODO: why do I need to use move? I don't understand
    HttpServer::new(move || {
        vec![
            app::create_monitoring_app().boxed(),
            app::create_avro_api_app().boxed(),
        ]
    })
    .bind(format!("127.0.0.1:{}", env::var("PORT")?))
    .unwrap()
    .shutdown_timeout(2)
    .start();

    sys.run();
    Ok(())
}

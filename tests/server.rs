use actix_rt::System;
use actix_web::{dev, App, HttpServer};

use avro_schema_registry::app;

use crate::common::settings::{get_host, get_port};

pub struct TestServer {
    sys: System,
    srv: dev::Server,
}

impl TestServer {
    pub fn start() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let sys = System::new("test");
            let srv = HttpServer::new(|| {
                App::new()
                    .configure(app::monitoring_routing)
                    .data(app::create_api_state())
                    .configure(app::api_routing)
            })
            .server_hostname("localhost")
            .disable_signals()
            .bind(format!("{}:{}", get_host(), get_port()))
            .unwrap()
            .start();
            let _ = tx.send((srv, actix_rt::System::current()));
            let _ = sys.run();
        });
        let (srv, sys) = rx.recv().unwrap();
        TestServer { sys, srv }
    }

    pub fn stop(self) {
        let _ = self.srv.stop(false);
        let _ = self.sys.stop();
    }
}

use std::{env, panic};

use actix_web::{http, test};
use diesel::pg::PgConnection;
use diesel::prelude::*;

use speculate::speculate;

use avro_schema_registry::app;

fn reset_global_config() {
    use avro_schema_registry::db::models::schema::configs::dsl::*;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    conn.transaction::<_, diesel::result::Error, _>(|| {
        diesel::update(configs)
            .filter(id.eq(0))
            .set(compatibility.eq("BACKWARD"))
            .execute(&conn)
    })
    .unwrap();
}

speculate! {
    before {
        use crate::common::AvroHeaders;

        reset_global_config();
        let mut srv = test::TestServer::with_factory(app::create_avro_api_app);
    }

    describe "get global config" {
        before {
            use actix_web::HttpMessage;
            use futures::future::Future;

            let request = srv
                .client(http::Method::GET, "/config")
                .with_avro_headers()
                .finish()
                .unwrap();
            let response = srv.execute(request.send()).unwrap();
            let expected = b"{\"compatibility\":\"BACKWARD\"}";
            let body = response.body().wait().unwrap();
        }

        it "returns BACKWARD" {
            assert!(response.status().is_success());
            assert_eq!(&body[..], expected);

        }
    }

    describe "set global config" {
        context "with valid compatibility FULL" {
            before {
                use actix_web::HttpMessage;
                use futures::future::Future;

                let request = srv
                    .client(http::Method::PUT, "/config")
                    .with_avro_headers()
                    .json(json!({"compatibility": "FULL"}))
                    .unwrap();
                let response = srv.execute(request.send()).unwrap();
                let expected = b"{\"compatibility\":\"FULL\"}";
                let body = response.body().wait().unwrap();
            }

            it "returns FULL" {
                assert!(response.status().is_success());
                assert_eq!(&body[..], expected);
            }
        }

        context "with invalid compatibility" {
            before {
                let request = srv
                    .client(http::Method::PUT, "/config")
                    .with_avro_headers()
                    .json(json!({"compatibility": "NOT_VALID"}))
                    .unwrap();
                let response = srv.execute(request.send()).unwrap();
            }

            it "returns 422" {
                assert!(response.status().is_client_error());
                assert_eq!(response.status().as_u16(), 422);
            }

            it "returns body with Invalid compatibility level" {
                use actix_web::HttpMessage;
                use futures::future::Future;

                let body = response.body().wait().unwrap();
                let body_str = std::str::from_utf8(&body[..]).unwrap();
                let expected = "{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}";
                assert_eq!(body_str, expected);
            }
        }
    }
}

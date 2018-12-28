use speculate::speculate;

use actix_web::{http::Method, test};

use crate::common::request::make_request;
use crate::db;

use avro_schema_registry::app;

speculate! {
    before {
        let conn = db::connection::connection();
        let mut srv = test::TestServer::with_factory(app::create_avro_api_app);

        db::config::reset_global(&conn);
    }

    after {
        db::cleanup::reset(&conn);
    }

    describe "get subjects" {
        before {
            db::subject::reset(&conn);
        }

        context "without subjects" {
            before {
                let response = make_request(
                    &mut srv,
                    Method::GET,
                    "subjects",
                    None,
                ).unwrap();
            }

            it "returns empty list" {
                assert!(response.status.is_success());
                assert_eq!(response.body, "[]");
            }
        }

        context "with subjects" {
            before {
                db::subject::add(&conn, vec![String::from("subject1"), String::from("subject2")]);
                let response = make_request(
                    &mut srv,
                    Method::GET,
                    "subjects",
                    None,
                ).unwrap();
            }

            it "returns list of subjects" {
                assert!(response.status.is_success());
                assert_eq!(response.body, "[\"subject1\",\"subject2\"]");
            }
        }
    }
}

use actix_web::http;
use serde_json;
use speculate::speculate;

use avro_schema_registry::api::SchemaBody;

use crate::common::request::TestRequest;
use crate::db;
use crate::server::TestServer;

speculate! {
    before {
        let conn = db::connection::connection();
        let server = TestServer::start();
    }

    after {
        server.stop();
        db::cleanup::reset(&conn);
    }

    describe "get schema" {
        before {
            db::subject::reset(&conn);
            db::schema::reset(&conn);
        }

        context "without schema" {
            it "returns empty list" {
                TestRequest::new(http::Method::GET, "/schemas/ids/1", None)
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40403,\"message\":\"Schema not found\"}")
                    .assert();
            }
        }

        context "with schema" {
            before {
                db::subject::add(&conn, vec![String::from("subject1")]);
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = db::subject::register_schema(&conn, String::from("subject1"), schema_s.to_string());
                let sch = SchemaBody{schema: schema_s};
            }

            it "returns schema" {
                TestRequest::new(http::Method::GET, &format!("/schemas/ids/{}", schema.id), None)
                         .expects_status(http::StatusCode::OK)
                         .expects_body(&serde_json::to_string(&sch).unwrap())
                         .assert();
                 }
        }
    }
}

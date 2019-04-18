use actix_web::http;
use serde_json;
use speculate::speculate;

use avro_schema_registry::api::SchemaBody;

use crate::common::server::{ApiTester, ApiTesterServer};
use crate::db;

speculate! {
    before {
        let conn = db::connection::connection();
        let server = ApiTesterServer::new();
    }

    after {
        db::cleanup::reset(&conn);
    }

    describe "get schema" {
        context "without schema" {
            it "returns empty list" {
                server.test(http::Method::GET, "/schemas/ids/1", None,
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40403,\"message\":\"Schema not found\"}");
            }
        }

        context "with schema" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = db::subject::register_schema(&conn, String::from("subject1"), schema_s.to_string());
                let sch = SchemaBody{schema: schema_s};
            }

            it "returns schema" {
                server.test(http::Method::GET, &format!("/schemas/ids/{}", schema.id), None,
                                 http::StatusCode::OK,
                                 &serde_json::to_string(&sch).unwrap());
                 }
        }
    }
}

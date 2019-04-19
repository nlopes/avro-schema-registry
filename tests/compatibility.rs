use actix_web::http;
use speculate::speculate;

use avro_schema_registry::api::SchemaBody;

use crate::common::server::{ApiTester, ApiTesterServer};
use crate::db;

speculate! {
    before {
        let conn = db::connection::connection();
        let server = ApiTesterServer::new();
        db::cleanup::reset(&conn);
    }

    after {
        db::cleanup::reset(&conn);
    }


    describe "test schema for compatibility" {
        before {
            let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
            let schema = SchemaBody{schema: schema_s};
        }

        context "with non existent subject" {
            it "returns 404 with 'Subject not found'" {
                server.test(http::Method::POST, "/compatibility/subjects/test.subject/versions/1",
                            Some(json!(schema)),
                            http::StatusCode::NOT_IMPLEMENTED,
                            "");
            }
        }

        context "with non existent version" {
            before {
                db::subject::create_test_subject_with_config(&conn, "FULL");
            }

            it "returns 404 with 'Version not found'" {
                server.test(http::Method::POST, "/compatibility/subjects/test.subject/versions/2",
                            Some(json!(schema)),
                            http::StatusCode::NOT_IMPLEMENTED,
                            "");
            }
        }
    }
}

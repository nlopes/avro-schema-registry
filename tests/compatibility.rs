use actix_web::http;
use speculate::speculate;

speculate! {
    before {
        use avro_schema_registry::db::{DbManage, DbPool};

        use crate::common::server::{ApiTesterServer};
        use crate::db::DbAuxOperations;

        let server = ApiTesterServer::new();
        let conn = DbPool::new_pool(Some(1)).connection().unwrap();
        conn.reset();
    }

    describe "test schema for compatibility" {
        before {
            use avro_schema_registry::api::SchemaBody;

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
                conn.create_test_subject_with_config("FULL");
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

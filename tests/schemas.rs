use actix_web::http;
use serde_json;
use speculate::speculate;

speculate! {
    before {
        use avro_schema_registry::db::{DbManage, DbPool};

        use crate::common::server::{ApiTesterServer};
        use crate::db::DbAuxOperations;

        let conn = DbPool::new_pool(Some(1)).connection().unwrap();
        let server = ApiTesterServer::new();
        conn.reset();
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
                use avro_schema_registry::api::SchemaBody;

                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = conn.register_schema(String::from("subject1"), schema_s.to_string());
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

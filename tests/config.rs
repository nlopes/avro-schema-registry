use actix_web::http;
use speculate::speculate;

speculate! {
    before {
        use avro_schema_registry::db::{DbManage, DbPool};
        use crate::common::server::ApiTesterServer;
        use crate::db::DbAuxOperations;

        let server = ApiTesterServer::new();
        let conn = DbPool::new_pool(Some(1)).connection().unwrap();
        conn.reset();
    }

    describe "get global config" {
        it "returns BACKWARD" {
            server.test(http::Method::GET, "/config", None,
                        http::StatusCode::OK, "{\"compatibility\":\"BACKWARD\"}");
        }
    }

    describe "set global config" {
        context "with valid compatibility FULL" {
            it "returns FULL" {
                server.test(http::Method::PUT, "/config", Some(json!({"compatibility": "FULL"})),
                            http::StatusCode::OK,
                            "{\"compatibility\":\"FULL\"}");
            }
        }

        context "with invalid compatibility" {
            it "returns 422 with Invalid compatibility level" {
                server.test(http::Method::PUT, "/config", Some(json!({"compatibility": "NOT_VALID"})),
                            http::StatusCode::UNPROCESSABLE_ENTITY,
                            "{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}");
            }
        }
    }

    describe "get compatibility level" {
        before {
            conn.create_test_subject_with_config("FULL");
        }
        context "existent subject" {
            it "returns valid compatibility" {
                server.test(http::Method::GET, "/config/test.subject", None,
                            http::StatusCode::OK, "{\"compatibility\":\"FULL\"}");
            }
        }

        context "non existent subject" {
            before {
                conn.reset_subjects();
            }

            it "returns 404 with Invalid compatibility level" {
                server.test(http::Method::GET, "/config/test.subject", None,
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40401,\"message\":\"Subject not found\"}");
            }
        }
    }

    describe "update compatibility level" {
        describe "existing subject" {
            before {
                conn.create_test_subject_with_config("FULL");
            }

            context "with valid compatibility FORWARD_TRANSITIVE" {
                it "returns FORWARD_TRANSITIVE" {
                    server.test(http::Method::PUT, "/config/test.subject",
                                Some(json!({"compatibility": "FORWARD_TRANSITIVE"})),
                                http::StatusCode::OK,
                                "{\"compatibility\":\"FORWARD_TRANSITIVE\"}");
                }
            }

            context "with invalid compatibility" {
                it "returns 422" {
                    server.test(http::Method::PUT, "/config/test.subject",
                                Some(json!({"compatibility": "NOT_VALID"})),
                                http::StatusCode::UNPROCESSABLE_ENTITY,
                                "{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}");
                }
            }
        }

        describe "non existing subject" {
            before {
                conn.reset_subjects();
            }

            context "with valid compatibility FULL" {
                it "returns 404" {
                    server.test(http::Method::PUT, "/config/test.subject",
                                Some(json!({"compatibility": "FULL"})),
                                http::StatusCode::NOT_FOUND,
                                "{\"error_code\":40401,\"message\":\"Subject not found\"}");
                }
            }
        }
    }
}

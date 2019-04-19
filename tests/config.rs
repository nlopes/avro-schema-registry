use actix_web::http;
use speculate::speculate;

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
        context "existent subject" {
            before {
                db::subject::create_test_subject_with_config(&conn, "FULL");
            }

            it "returns valid compatibility" {
                server.test(http::Method::GET, "/config/test.subject", None,
                            http::StatusCode::OK, "{\"compatibility\":\"FULL\"}");
            }
        }

        context "non existent subject" {
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
                db::subject::create_test_subject_with_config(&conn, "FULL");
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

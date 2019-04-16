use actix_web::http;
use speculate::speculate;

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


    describe "get global config" {
        it "returns BACKWARD" {
            TestRequest::new(http::Method::GET, "/config", None)
                .expects_status(http::StatusCode::OK)
                .expects_body("{\"compatibility\":\"BACKWARD\"}")
                .assert();
        }
    }

    describe "set global config" {
        context "with valid compatibility FULL" {
            it "returns FULL" {
                TestRequest::new(http::Method::PUT, "/config", Some(json!({"compatibility": "FULL"})))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("{\"compatibility\":\"FULL\"}")
                    .assert();
            }
        }

        context "with invalid compatibility" {
            it "returns 422 with Invalid compatibility level" {
                TestRequest::new(http::Method::PUT, "/config", Some(json!({"compatibility": "NOT_VALID"})))
                    .expects_status(http::StatusCode::UNPROCESSABLE_ENTITY)
                    .expects_body("{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}")
                    .assert();
            }
        }
    }

    describe "get compatibility level" {
        context "existent subject" {
            before {
                db::subject::create_test_subject_with_config(&conn, "FULL");
            }

            it "returns valid compatibility" {
                TestRequest::new(http::Method::GET, "/config/test.subject", None)
                    .expects_status(http::StatusCode::OK)
                    .expects_body("{\"compatibility\":\"FULL\"}")
                    .assert();
            }
        }

        context "non existent subject" {
            it "returns 404 with Invalid compatibility level" {
                TestRequest::new(http::Method::GET, "/config/test.subject", None)
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                    .assert();
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
                    TestRequest::new(http::Method::PUT,
                                     "/config/test.subject",
                                     Some(json!({"compatibility": "FORWARD_TRANSITIVE"})))
                        .expects_status(http::StatusCode::OK)
                        .expects_body("{\"compatibility\":\"FORWARD_TRANSITIVE\"}")
                        .assert();
                }
            }

            context "with invalid compatibility" {
                it "returns 422" {
                    TestRequest::new(http::Method::PUT,
                                     "/config/test.subject",
                                     Some(json!({"compatibility": "NOT_VALID"})))
                        .expects_status(http::StatusCode::UNPROCESSABLE_ENTITY)
                        .expects_body("{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}")
                        .assert();
                }
            }
        }

        describe "non existing subject" {
            context "with valid compatibility FULL" {
                it "returns 404" {
                    TestRequest::new(http::Method::PUT,
                                     "/config/test.subject",
                                     Some(json!({"compatibility": "FULL"})))
                        .expects_status(http::StatusCode::NOT_FOUND)
                        .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                        .assert();
                }
            }
        }
    }
}

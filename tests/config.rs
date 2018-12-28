use std::panic;

use actix_web::{http::Method, test};
use speculate::speculate;

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

    describe "get global config" {
        before {
            let response = make_request(
                &mut srv,
                Method::GET,
                "config",
                None,
                ).unwrap();
        }

        it "returns BACKWARD" {
            assert!(response.status.is_success());
            let expected = "{\"compatibility\":\"BACKWARD\"}";
            assert_eq!(response.body, expected);
        }
    }

    describe "set global config" {
        context "with valid compatibility FULL" {
            before {
                let response = make_request(
                    &mut srv,
                    Method::PUT,
                    "config",
                    Some(json!({"compatibility": "FULL"})),
                ).unwrap();
            }

            it "returns FULL" {
                assert!(response.status.is_success());
                let expected = "{\"compatibility\":\"FULL\"}";
                assert_eq!(response.body, expected);
            }
        }

        context "with invalid compatibility" {
            before {
                let response = make_request(
                    &mut srv,
                    Method::PUT,
                    "config",
                    Some(json!({"compatibility": "NOT_VALID"})),
                ).unwrap();
            }

            it "returns 422" {
                assert!(response.status.is_client_error());
                assert_eq!(response.status.as_u16(), 422);
            }

            it "returns body with Invalid compatibility level" {
                let expected = "{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}";
                assert_eq!(response.body, expected);
            }
        }
    }

    describe "get compatibility level" {
        context "existent subject" {
            before {
                db::subject::create_test_subject_with_config(&conn, "FULL");
                let response = make_request(
                    &mut srv,
                    Method::GET,
                    "config/test.subject",
                    None,
                ).unwrap();
            }

            it "returns valid compatibility" {
                assert!(response.status.is_success());
                let expected = "{\"compatibility\":\"FULL\"}";
                assert_eq!(response.body, expected);
            }
        }

        context "non existent subject" {
            before {
                let response = make_request(
                    &mut srv,
                    Method::GET,
                    "config/test.subject",
                    None,
                ).unwrap();
            }

            it "returns 404" {
                assert!(response.status.is_client_error());
                assert_eq!(response.status.as_u16(), 404);
            }

            it "returns body with Invalid compatibility level" {
                let expected = "{\"error_code\":40401,\"message\":\"Subject not found\"}";
                assert_eq!(response.body, expected);
            }
        }
    }

    describe "update compatibility level" {
        describe "existing subject" {
            before {
                db::subject::create_test_subject_with_config(&conn, "FULL");
            }

            context "with valid compatibility FORWARD_TRANSITIVE" {
                before {
                    let response = make_request(
                        &mut srv,
                        Method::PUT,
                        "config/test.subject",
                        Some(json!({"compatibility": "FORWARD_TRANSITIVE"})),
                    )
                    .unwrap();
                }

                it "returns FORWARD_TRANSITIVE" {
                    assert!(response.status.is_success());
                    let expected = "{\"compatibility\":\"FORWARD_TRANSITIVE\"}";
                    assert_eq!(response.body, expected);
                }
            }

            context "with invalid compatibility" {
                before {
                    let response = make_request(
                        &mut srv,
                        Method::PUT,
                        "config/test.subject",
                        Some(json!({"compatibility": "NOT_VALID"})),
                    )
                    .unwrap();
                }

                it "returns 422" {
                    assert!(response.status.is_client_error());
                    assert_eq!(response.status.as_u16(), 422);
                }

                it "returns body with Invalid compatibility level" {
                    let expected = "{\"error_code\":42203,\"message\":\"Invalid compatibility level\"}";
                    assert_eq!(response.body, expected);
                }
            }
        }

        describe "non existing subject" {
            context "with valid compatibility FULL" {
                before {
                    let response = make_request(
                        &mut srv,
                        Method::PUT,
                        "config/test.subject",
                        Some(json!({"compatibility": "FULL"})),
                    )
                    .unwrap();
                }

                it "returns 404" {
                    assert!(response.status.is_client_error());
                    assert_eq!(response.status.as_u16(), 404);
                }

                it "returns body with Invalid compatibility level" {
                    let expected = "{\"error_code\":40401,\"message\":\"Subject not found\"}";
                    assert_eq!(response.body, expected);
                }
            }
        }
    }
}

use actix_web::http;
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

    describe "get subjects" {
        before {
            db::subject::reset(&conn);
        }

        context "without subjects" {
            it "returns empty list" {
                TestRequest::new(http::Method::GET, "/subjects", None)
                    .expects_status(http::StatusCode::OK)
                    .expects_body("[]")
                    .assert();
            }
        }

        context "with subjects" {
            before {
                db::subject::add(&conn, vec![String::from("subject1"), String::from("subject2")]);
            }

            it "returns list of subjects" {
                TestRequest::new(http::Method::GET, "/subjects", None)
                    .expects_status(http::StatusCode::OK)
                    .expects_body("[\"subject1\",\"subject2\"]")
                    .assert();
            }
        }
    }

    describe "get versions under subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                TestRequest::new(http::Method::GET, "/subjects/test.subject/versions", None)
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                    .assert();
            }
        }

        context "without versions" {
            before {
                db::subject::add(&conn, vec![String::from("test.subject")]);
            }

            after {
                db::cleanup::reset(&conn);
            }

            // This should never happen with correct usage of the API
            it "returns 404 with 'Subject not found'" {
                TestRequest::new(http::Method::GET, "/subjects/test.subject/versions", None)
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                    .assert();
            }
        }

        context "with versions" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("")
                    .assert();
            }

            it "returns list of one" {
                TestRequest::new(http::Method::GET, "/subjects/test.subject/versions", None)
                    .expects_status(http::StatusCode::OK)
                    .expects_body("[1]") // TODO(nlopes): dangerous, postgresql can pick any other ID
                    .assert();
            }

            it "returns list of many" {
                let schema2_s = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();
                let schema2 = SchemaBody{schema: schema2_s};
                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema2)))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("")
                    .assert();

                TestRequest::new(http::Method::GET, "/subjects/test.subject/versions", None)
                    .expects_status(http::StatusCode::OK)
                    .expects_body("[1,2]")
                    .assert();
            }
        }
    }

    describe "delete subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                TestRequest::new(http::Method::DELETE, "/subjects/test.subject", None)
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                    .assert();
            }
        }

        context "with subject" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("")
                    .assert();
            }

            it "returns list with versions of schemas deleted" {
                TestRequest::new(http::Method::DELETE, "/subjects/test.subject", None)
                    .expects_status(http::StatusCode::OK)
                    .expects_body("[1]")
                    .assert();
            }
        }
    }

    describe "get version of schema registered under subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                TestRequest::new(http::Method::GET, "/subjects/test.subject/versions/1", None)
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                    .assert();
            }
        }

        context "with subject" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("")
                    .assert();
            }

            context "with non existing version" {
                it "returns 404 with 'Version not found'" {
                    TestRequest::new(http::Method::GET, "/subjects/test.subject/versions/2", None)
                        .expects_status(http::StatusCode::NOT_FOUND)
                        .expects_body("{\"error_code\":40402,\"message\":\"Version not found\"}")
                        .assert();
                }
            }

            context "with version out of bounds" {
                it "returns 422 with 'Invalid version' for lower bound" {
                    TestRequest::new(http::Method::GET, "/subjects/test.subject/versions/0", None)
                        .expects_status(http::StatusCode::UNPROCESSABLE_ENTITY)
                        .expects_body("{\"error_code\":42202,\"message\":\"Invalid version\"}")
                        .assert();
                }

                it "returns 422 with 'Invalid version' for upper bound" {
                    TestRequest::new(http::Method::GET, "/subjects/test.subject/versions/2147483648", None)
                        .expects_status(http::StatusCode::UNPROCESSABLE_ENTITY)
                        .expects_body("{\"error_code\":42202,\"message\":\"Invalid version\"}")
                        .assert();
                }
            }

            context "with latest version" {
                it "returns version with schema" {
                    // TODO(nlopes): fix expect body - requires knowing that the id
                    // below is 86 - maybe direct sql query
                    //
                    //"{\"subject\":\"test.subject\",\"id\":86,\"version\":1,\"schema\":\"{\\n    \\\"type\\\": \\\"record\\\",\\n    \\\"name\\\": \\\"test\\\",\\n    \\\"fields\\\":\\n    [\\n        {\\n            \\\"type\\\": \\\"string\\\",\\n             \\\"name\\\": \\\"field1\\\"\\n           },\\n           {\\n             \\\"type\\\": \\\"int\\\",\\n             \\\"name\\\": \\\"field2\\\"\\n           }\\n         ]\\n}\\n\"}";

                    TestRequest::new(http::Method::GET, "/subjects/test.subject/versions/latest", None)
                        .expects_status(http::StatusCode::OK)
                        .expects_body("")
                        .assert();
                }
            }

            context "with existing version" {
                it "returns version with schema" {
                    // TODO(nlopes): fix expect body - requires knowing that the id
                    // below is 86 - maybe direct sql query
                    //
                    //"{\"subject\":\"test.subject\",\"id\":86,\"version\":1,\"schema\":\"{\\n    \\\"type\\\": \\\"record\\\",\\n    \\\"name\\\": \\\"test\\\",\\n    \\\"fields\\\":\\n    [\\n        {\\n            \\\"type\\\": \\\"string\\\",\\n             \\\"name\\\": \\\"field1\\\"\\n           },\\n           {\\n             \\\"type\\\": \\\"int\\\",\\n             \\\"name\\\": \\\"field2\\\"\\n           }\\n         ]\\n}\\n\"}";

                    TestRequest::new(http::Method::GET, "/subjects/test.subject/versions/1", None)
                        .expects_status(http::StatusCode::OK)
                        .expects_body("")
                        .assert();
                }
            }
        }
    }

    describe "register schema under subject" {
        context "with valid schema" {
            it "returns schema identifier" {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};

                // TODO(nlopes): Check for body "{\"id\":\"147\"}"
                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("")
                    .assert();
            }
        }

        context "with invalid schema" {
            it "returns 422 with 'Invalid Avro schema'" {
                let schema = SchemaBody{schema: "{}".to_string()};

                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)))
                    .expects_status(http::StatusCode::UNPROCESSABLE_ENTITY)
                        .expects_body("{\"error_code\":42201,\"message\":\"Invalid Avro schema\"}")
                    .assert();
            }
        }
    }

    describe "check schema registration" {
        before {
            let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
            let schema = SchemaBody{schema: schema_s};
        }

        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                TestRequest::new(http::Method::POST, "/subjects/test.subject", Some(json!(schema)))
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                    .assert();
            }
        }

        context "with subject but different schema" {
            before {
                let schema2_s = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();
                let schema2 = SchemaBody{schema: schema2_s};
                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema2)))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("")
                    .assert();
            }

            it "returns 404 with Subject not found" {
                TestRequest::new(http::Method::POST, "/subjects/test.subject", Some(json!(schema)))
                     .expects_status(http::StatusCode::NOT_FOUND)
                     .expects_body("{\"error_code\":40403,\"message\":\"Schema not found\"}")
                     .assert();
            }
        }
    }

    describe "delete schema version under subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                TestRequest::new(http::Method::DELETE, "/subjects/test.subject/versions/1", None)
                    .expects_status(http::StatusCode::NOT_FOUND)
                    .expects_body("{\"error_code\":40401,\"message\":\"Subject not found\"}")
                    .assert();
            }
        }

        context "with subject" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                TestRequest::new(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)))
                    .expects_status(http::StatusCode::OK)
                    .expects_body("")
                    .assert();
            }

            context "with non existing version" {
                it "returns 404 with 'Version not found'" {
                    TestRequest::new(http::Method::DELETE, "/subjects/test.subject/versions/2", None)
                        .expects_status(http::StatusCode::NOT_FOUND)
                        .expects_body("{\"error_code\":40402,\"message\":\"Version not found\"}")
                        .assert();
                }
            }

            context "with version out of bounds" {
                it "returns 422 with 'Invalid version'" {
                    TestRequest::new(http::Method::DELETE, "/subjects/test.subject/versions/0", None)
                        .expects_status(http::StatusCode::UNPROCESSABLE_ENTITY)
                        .expects_body("{\"error_code\":42202,\"message\":\"Invalid version\"}")
                        .assert();
                }
            }

            context "with existing version" {
                it "returns list with versions of schemas deleted" {
                    TestRequest::new(http::Method::DELETE, "/subjects/test.subject/versions/1", None)
                        .expects_status(http::StatusCode::OK)
                        .expects_body("1")
                        .assert();
                }
            }
        }
    }
}

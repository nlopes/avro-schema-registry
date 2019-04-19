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

    describe "get subjects" {
        before {
            db::subject::reset(&conn);
        }

        context "without subjects" {
            it "returns empty list" {
                server.test(http::Method::GET, "/subjects", None, http::StatusCode::OK, "[]");
            }
        }

        context "with subjects" {
            before {
                db::subject::add(&conn, vec![String::from("subject1"), String::from("subject2")]);
            }

            it "returns list of subjects" {
                server.test(http::Method::GET, "/subjects", None,
                            http::StatusCode::OK,
                            "[\"subject1\",\"subject2\"]");
            }
        }

    }

    describe "get versions under subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                server.test(http::Method::GET, "/subjects/test.subject/versions", None,
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40401,\"message\":\"Subject not found\"}");
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
                server.test(http::Method::GET, "/subjects/test.subject/versions", None,
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40401,\"message\":\"Subject not found\"}");
            }
        }

        context "with versions" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)),
                            http::StatusCode::OK, "");
            }

            it "returns list of one" {
                // TODO(nlopes): dangerous, postgresql can pick any other ID
                server.test(http::Method::GET, "/subjects/test.subject/versions", None,
                    http::StatusCode::OK, "[1]");
            }

            it "returns list of many" {
                let schema2_s = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();
                let schema2 = SchemaBody{schema: schema2_s};

                // This modifies the database state in preparation for the next request
                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema2)),
                    http::StatusCode::OK, "");

                server.test(http::Method::GET, "/subjects/test.subject/versions", None,
                            http::StatusCode::OK, "[1,2]");
            }
        }
    }

    describe "delete subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                server.test(http::Method::DELETE, "/subjects/test.subject", None,
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40401,\"message\":\"Subject not found\"}");
            }
        }

        context "with subject" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)),
                            http::StatusCode::OK, "");
            }

            it "returns list with versions of schemas deleted" {
                server.test(http::Method::DELETE, "/subjects/test.subject", None,
                            http::StatusCode::OK, "[1]");
            }
        }
    }

    describe "get version of schema registered under subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                server.test(http::Method::GET, "/subjects/test.subject/versions/1", None,
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40401,\"message\":\"Subject not found\"}");
            }
        }

        context "with subject" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)),
                            http::StatusCode::OK, "");
            }

            context "with non existing version" {
                it "returns 404 with 'Version not found'" {
                    server.test(http::Method::GET, "/subjects/test.subject/versions/2", None,
                                http::StatusCode::NOT_FOUND,
                                "{\"error_code\":40402,\"message\":\"Version not found\"}");
                }
            }

            context "with version out of bounds" {
                it "returns 422 with 'Invalid version' for lower bound" {
                    server.test(http::Method::GET, "/subjects/test.subject/versions/0", None,
                                http::StatusCode::UNPROCESSABLE_ENTITY,
                                "{\"error_code\":42202,\"message\":\"Invalid version\"}");
                }

                it "returns 422 with 'Invalid version' for upper bound" {
                    server.test(http::Method::GET, "/subjects/test.subject/versions/2147483648", None,
                                http::StatusCode::UNPROCESSABLE_ENTITY,
                                "{\"error_code\":42202,\"message\":\"Invalid version\"}");
                }
            }

            context "with latest version" {
                it "returns version with schema" {
                    // TODO(nlopes): fix expect body - requires knowing that the id
                    // below is 86 - maybe direct sql query
                    //
                    //"{\"subject\":\"test.subject\",\"id\":86,\"version\":1,\"schema\":\"{\\n    \\\"type\\\": \\\"record\\\",\\n    \\\"name\\\": \\\"test\\\",\\n    \\\"fields\\\":\\n    [\\n        {\\n            \\\"type\\\": \\\"string\\\",\\n             \\\"name\\\": \\\"field1\\\"\\n           },\\n           {\\n             \\\"type\\\": \\\"int\\\",\\n             \\\"name\\\": \\\"field2\\\"\\n           }\\n         ]\\n}\\n\"}";

                    server.test(http::Method::GET, "/subjects/test.subject/versions/latest", None,
                                http::StatusCode::OK, "");
                }
            }

            context "with existing version" {
                it "returns version with schema" {
                    // TODO(nlopes): fix expect body - requires knowing that the id
                    // below is 86 - maybe direct sql query
                    //
                    //"{\"subject\":\"test.subject\",\"id\":86,\"version\":1,\"schema\":\"{\\n    \\\"type\\\": \\\"record\\\",\\n    \\\"name\\\": \\\"test\\\",\\n    \\\"fields\\\":\\n    [\\n        {\\n            \\\"type\\\": \\\"string\\\",\\n             \\\"name\\\": \\\"field1\\\"\\n           },\\n           {\\n             \\\"type\\\": \\\"int\\\",\\n             \\\"name\\\": \\\"field2\\\"\\n           }\\n         ]\\n}\\n\"}";

                    server.test(http::Method::GET, "/subjects/test.subject/versions/1", None,
                                http::StatusCode::OK, "");

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
                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)),
                            http::StatusCode::OK, "");
            }
        }

        context "with invalid schema" {
            it "returns 422 with 'Invalid Avro schema'" {
                let schema = SchemaBody{schema: "{}".to_string()};

                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)),
                            http::StatusCode::UNPROCESSABLE_ENTITY,
                            "{\"error_code\":42201,\"message\":\"Invalid Avro schema\"}");
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
                server.test(http::Method::POST, "/subjects/test.subject", Some(json!(schema)),
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40401,\"message\":\"Subject not found\"}");
            }
        }

        context "with subject but different schema" {
            before {
                let schema2_s = std::fs::read_to_string("tests/fixtures/schema2.json").unwrap();
                let schema2 = SchemaBody{schema: schema2_s};
                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema2)),
                            http::StatusCode::OK, "");
            }

            it "returns 404 with Subject not found" {
                server.test(http::Method::POST, "/subjects/test.subject", Some(json!(schema)),
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40403,\"message\":\"Schema not found\"}");
            }
        }
    }

    describe "delete schema version under subject" {
        context "without subject" {
            it "returns 404 with 'Subject not found'" {
                server.test(http::Method::DELETE, "/subjects/test.subject/versions/1", None,
                            http::StatusCode::NOT_FOUND,
                            "{\"error_code\":40401,\"message\":\"Subject not found\"}");
            }
        }

        context "with subject" {
            before {
                let schema_s = std::fs::read_to_string("tests/fixtures/schema.json").unwrap();
                let schema = SchemaBody{schema: schema_s};
                server.test(http::Method::POST, "/subjects/test.subject/versions", Some(json!(schema)),
                    http::StatusCode::OK, "");
            }

            context "with non existing version" {
                it "returns 404 with 'Version not found'" {
                    server.test(http::Method::DELETE, "/subjects/test.subject/versions/2", None,
                                http::StatusCode::NOT_FOUND,
                                "{\"error_code\":40402,\"message\":\"Version not found\"}");
                }
            }

            context "with version out of bounds" {
                it "returns 422 with 'Invalid version'" {
                    server.test(http::Method::DELETE, "/subjects/test.subject/versions/0", None,
                                http::StatusCode::UNPROCESSABLE_ENTITY,
                                "{\"error_code\":42202,\"message\":\"Invalid version\"}");
                }
            }

            context "with existing version" {
                it "returns list with versions of schemas deleted" {
                    server.test(http::Method::DELETE, "/subjects/test.subject/versions/1", None,
                                http::StatusCode::OK, "1");
                }
            }
        }
    }
}

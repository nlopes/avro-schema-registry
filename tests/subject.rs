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
}

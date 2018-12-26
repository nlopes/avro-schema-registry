use actix_web::{client::ClientRequestBuilder, http};

pub trait AvroHeaders {
    fn with_avro_headers(&mut self) -> &mut ClientRequestBuilder;
}

impl AvroHeaders for ClientRequestBuilder {
    fn with_avro_headers(&mut self) -> &mut Self {
        self.header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::ACCEPT, "application/vnd.schemaregistry+json")
            .header(http::header::AUTHORIZATION, "Basic OmZhaAo=")
    }
}

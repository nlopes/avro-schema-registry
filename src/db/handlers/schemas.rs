use actix::Handler;

use crate::api::errors::ApiError;

use super::{ConnectionPooler, GetSchema, Schema, SchemaResponse};

impl Handler<GetSchema> for ConnectionPooler {
    type Result = Result<SchemaResponse, ApiError>;

    fn handle(&mut self, schema_query: GetSchema, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Schema::get_json_by_id(schema_query.id, &conn)
            .and_then(|json| Ok(SchemaResponse { schema: json }))
    }
}

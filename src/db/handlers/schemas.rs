use actix::Handler;

use crate::api::errors::ApiError;

use super::{ConnectionPooler, DeleteSchemaVersion, GetSchema, Schema, SchemaResponse};

impl Handler<GetSchema> for ConnectionPooler {
    type Result = Result<SchemaResponse, ApiError>;

    fn handle(&mut self, schema_query: GetSchema, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Schema::get_json_by_id(schema_query.id, &conn)
            .and_then(|json| Ok(SchemaResponse { schema: json }))
    }
}

impl Handler<DeleteSchemaVersion> for ConnectionPooler {
    type Result = Result<i32, ApiError>;

    fn handle(
        &mut self,
        delete_schema_version: DeleteSchemaVersion,
        _: &mut Self::Context,
    ) -> Self::Result {
        let conn = self.connection()?;
        delete_schema_version.execute(&conn)
    }
}

use actix::Handler;

use crate::api::errors::ApiError;

use super::{
    ConnectionPooler, DeleteSchemaVersion, GetSchema, RegisterSchema, RegisterSchemaResponse,
    Schema, SchemaResponse, SchemaVersion,
};

impl Handler<GetSchema> for ConnectionPooler {
    type Result = Result<SchemaResponse, ApiError>;

    fn handle(&mut self, schema_query: GetSchema, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Schema::get_json_by_id(&conn, schema_query.id)
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
        SchemaVersion::delete_version_with_subject(&conn, delete_schema_version)
    }
}

impl Handler<RegisterSchema> for ConnectionPooler {
    type Result = Result<RegisterSchemaResponse, ApiError>;
    fn handle(&mut self, data: RegisterSchema, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        let schema = Schema::register_new_version(&conn, data)?;

        Ok(RegisterSchemaResponse {
            id: format!("{}", schema.id),
        })
    }
}

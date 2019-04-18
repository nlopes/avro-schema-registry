use actix::Handler;

use crate::api::errors::{ApiError, ApiErrorCode};

use super::{
    ConnectionPooler, DeleteSchemaVersion, GetSchema, GetSubjectVersionResponse, RegisterSchema,
    RegisterSchemaResponse, Schema, SchemaResponse, SchemaVersion, VerifySchemaRegistration,
};

impl Handler<GetSchema> for ConnectionPooler {
    type Result = Result<SchemaResponse, ApiError>;

    fn handle(&mut self, schema_query: GetSchema, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Schema::get_by_id(&conn, schema_query.id).map(|schema| SchemaResponse {
            schema: schema.json,
        })
    }
}

impl Handler<DeleteSchemaVersion> for ConnectionPooler {
    type Result = Result<u32, ApiError>;

    fn handle(
        &mut self,
        delete_schema_version: DeleteSchemaVersion,
        _: &mut Self::Context,
    ) -> Self::Result {
        use crate::api::version::VersionLimit;

        let conn = self.connection()?;
        if !delete_schema_version.version.within_limits() {
            return Err(ApiError::new(ApiErrorCode::InvalidVersion));
        }
        SchemaVersion::delete_version_with_subject(&conn, delete_schema_version)
    }
}

impl Handler<RegisterSchema> for ConnectionPooler {
    type Result = Result<RegisterSchemaResponse, ApiError>;
    fn handle(&mut self, data: RegisterSchema, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Schema::register_new_version(&conn, data).map(|schema| RegisterSchemaResponse {
            id: format!("{}", schema.id),
        })
    }
}

impl Handler<VerifySchemaRegistration> for ConnectionPooler {
    type Result = Result<GetSubjectVersionResponse, ApiError>;

    fn handle(&mut self, verify: VerifySchemaRegistration, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Schema::verify_registration(&conn, verify.subject, verify.schema)
    }
}

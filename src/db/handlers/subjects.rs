use actix::Handler;

use crate::api::errors::{ApiError, ApiErrorCode};

use super::{
    ConnectionPooler, DeleteSubject, DeleteSubjectResponse, GetSubjectVersion,
    GetSubjectVersionResponse, GetSubjectVersions, GetSubjects, SchemaVersion, Subject,
    SubjectList, SubjectVersionsResponse,
};

impl Handler<GetSubjects> for ConnectionPooler {
    type Result = Result<SubjectList, ApiError>;

    fn handle(&mut self, _: GetSubjects, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Subject::distinct_names(&conn).map(|content| SubjectList { content })
    }
}

impl Handler<GetSubjectVersions> for ConnectionPooler {
    type Result = Result<SubjectVersionsResponse, ApiError>;

    fn handle(&mut self, subject_query: GetSubjectVersions, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        SchemaVersion::versions_with_subject_name(&conn, subject_query.subject)
            .map(|versions| SubjectVersionsResponse { versions })
    }
}

impl Handler<DeleteSubject> for ConnectionPooler {
    type Result = Result<DeleteSubjectResponse, ApiError>;

    fn handle(&mut self, query: DeleteSubject, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Subject::delete_by_name(&conn, query.subject)
            .map(|versions| DeleteSubjectResponse { versions })
    }
}

impl Handler<GetSubjectVersion> for ConnectionPooler {
    type Result = Result<GetSubjectVersionResponse, ApiError>;

    fn handle(&mut self, query: GetSubjectVersion, _: &mut Self::Context) -> Self::Result {
        use crate::api::version::VersionLimit;

        let conn = self.connection()?;
        match query.version {
            Some(v) => {
                if !v.within_limits() {
                    return Err(ApiError::new(ApiErrorCode::InvalidVersion));
                }
                SchemaVersion::get_schema_id(&conn, query.subject.to_string(), v)
            }
            None => SchemaVersion::get_schema_id_from_latest(&conn, query.subject.to_string()),
        }
        .map(|o| GetSubjectVersionResponse {
            subject: query.subject.to_string(),
            id: o.0,
            version: o.1,
            schema: o.2,
        })
    }
}

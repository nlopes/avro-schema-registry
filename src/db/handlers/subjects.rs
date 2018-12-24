use actix::Handler;
use diesel::prelude::*;

use crate::api::errors::{ApiError, ApiErrorCode};

use super::{
    ConnectionPooler, DeleteSubject, DeleteSubjectResponse, GetSubjectVersion,
    GetSubjectVersionResponse, GetSubjectVersions, GetSubjects, RegisterSchema,
    RegisterSchemaResponse, SubjectList, SubjectVersionsResponse, VerifySchemaRegistration,
};

impl Handler<GetSubjects> for ConnectionPooler {
    type Result = Result<SubjectList, ApiError>;

    fn handle(&mut self, _: GetSubjects, _: &mut Self::Context) -> Self::Result {
        use super::schema::subjects::dsl::{name, subjects as db_subjects};

        let conn = self.connection()?;

        db_subjects.select(name).load::<String>(&conn).map_or_else(
            |_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
            |subjects| Ok(SubjectList { content: subjects }),
        )
    }
}

impl Handler<GetSubjectVersions> for ConnectionPooler {
    type Result = Result<SubjectVersionsResponse, ApiError>;

    fn handle(&mut self, subject_query: GetSubjectVersions, _: &mut Self::Context) -> Self::Result {
        use super::schema::schema_versions::dsl::*;
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        let conn = self.connection()?;
        schema_versions
            .inner_join(subjects.on(subject_id.eq(subjects_id)))
            .filter(name.eq(&subject_query.subject))
            .select(version)
            .load::<Option<i32>>(&conn)
            .map_or_else(
                |_| Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                |res| {
                    if res.len() == 0 {
                        Err(ApiError::new(ApiErrorCode::SubjectNotFound))
                    } else {
                        Ok(SubjectVersionsResponse { versions: res })
                    }
                },
            )
    }
}

impl Handler<DeleteSubject> for ConnectionPooler {
    type Result = Result<DeleteSubjectResponse, ApiError>;

    fn handle(&mut self, query: DeleteSubject, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        DeleteSubject::delete(query.subject, &conn)
    }
}

impl Handler<GetSubjectVersion> for ConnectionPooler {
    type Result = Result<GetSubjectVersionResponse, ApiError>;

    fn handle(&mut self, query: GetSubjectVersion, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        query.execute(&conn)
    }
}

impl Handler<RegisterSchema> for ConnectionPooler {
    type Result = Result<RegisterSchemaResponse, ApiError>;
    fn handle(&mut self, data: RegisterSchema, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        let schema = data.find_schema(&conn)?;
        let schema_id = match schema {
            Some(s) => {
                // TODO
                // if I can find a version for this schema, then
                s.id
            }
            None => {
                let sc = data.create_new_schema(&conn).ok().unwrap();
                println!("No schema, should have created {:?}", sc);
                sc.id
            }
        };

        Ok(RegisterSchemaResponse {
            id: format!("{}", schema_id),
        })
    }
}

impl Handler<VerifySchemaRegistration> for ConnectionPooler {
    type Result = Result<GetSubjectVersionResponse, ApiError>;

    fn handle(&mut self, verify: VerifySchemaRegistration, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        verify.execute(&conn)
    }
}

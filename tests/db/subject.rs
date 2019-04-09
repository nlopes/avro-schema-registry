use diesel::prelude::*;

use avro_schema_registry::api::errors::ApiError;
use avro_schema_registry::db::models::Subject;

pub fn create_test_subject_with_config(conn: &super::PgConnection, compat: &str) {
    use avro_schema_registry::db::models::schema::configs::dsl::{
        compatibility, configs, created_at as config_created_at, subject_id,
        updated_at as config_updated_at,
    };
    use avro_schema_registry::db::models::schema::subjects::dsl::*;

    conn.transaction::<_, diesel::result::Error, _>(|| {
        diesel::insert_into(subjects)
            .values((
                name.eq("test.subject"),
                created_at.eq(diesel::dsl::now),
                updated_at.eq(diesel::dsl::now),
            ))
            .get_result::<Subject>(conn)
            .and_then(|subject| {
                diesel::insert_into(configs)
                    .values((
                        compatibility.eq(compat),
                        config_created_at.eq(diesel::dsl::now),
                        config_updated_at.eq(diesel::dsl::now),
                        subject_id.eq(subject.id),
                    ))
                    .execute(conn)
            })
    })
    .unwrap();
}

pub fn add(conn: &super::PgConnection, subjects: Vec<String>) {
    conn.transaction::<String, ApiError, _>(|| {
        Ok(subjects
            .into_iter()
            .map(|subject| {
                Subject::insert(conn, subject)
                    .expect("could not insert subject")
                    .name
            })
            .collect())
    })
    .unwrap();
}

pub fn reset(conn: &super::PgConnection) {
    use avro_schema_registry::db::models::schema::subjects::dsl::*;
    diesel::delete(subjects).execute(conn).unwrap();
}

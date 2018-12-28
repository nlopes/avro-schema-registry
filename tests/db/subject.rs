use diesel::prelude::*;

use super::connection;
use avro_schema_registry::db::models::Subject;

pub fn create_test_subject_with_config(compat: &str) {
    use avro_schema_registry::db::models::schema::configs::dsl::{
        compatibility, configs, created_at as config_created_at, subject_id,
        updated_at as config_updated_at,
    }; ;
    use avro_schema_registry::db::models::schema::subjects::dsl::*;

    let conn = connection::connection();

    conn.transaction::<_, diesel::result::Error, _>(|| {
        diesel::insert_into(subjects)
            .values((
                name.eq("test.subject"),
                created_at.eq(diesel::dsl::now),
                updated_at.eq(diesel::dsl::now),
            ))
            .get_result::<Subject>(&conn)
            .and_then(|subject| {
                diesel::insert_into(configs)
                    .values((
                        compatibility.eq(compat),
                        config_created_at.eq(diesel::dsl::now),
                        config_updated_at.eq(diesel::dsl::now),
                        subject_id.eq(subject.id),
                    ))
                    .execute(&conn)
            })
    })
    .unwrap();
}

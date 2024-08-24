use diesel::prelude::*;

use avro_schema_registry::db::models::Schema;
use avro_schema_registry::db::DbConnection;

pub trait DbAuxOperations {
    fn reset(&mut self);
    fn reset_subjects(&mut self);

    // These two are not yet used anywhere but I wanted them as part of the trait and
    // implemented
    #[allow(dead_code)]
    fn reset_schemas(&mut self);
    #[allow(dead_code)]
    fn reset_configs_global(&mut self);

    fn create_test_subject_with_config(&mut self, compat: &str);
    fn add_subjects(&mut self, subjects: Vec<String>);
    fn register_schema(&mut self, subject: String, schema: String) -> Schema;
}

impl DbAuxOperations for DbConnection {
    fn reset(&mut self) {
        use avro_schema_registry::db::models::schema::configs::dsl::configs;
        use avro_schema_registry::db::models::schema::schema_versions::dsl::schema_versions;
        use avro_schema_registry::db::models::schema::schemas::dsl::schemas;
        use avro_schema_registry::db::models::schema::subjects::dsl::subjects;

        self.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(configs).execute(conn)?;
            diesel::delete(schemas).execute(conn)?;
            diesel::delete(subjects).execute(conn)?;
            diesel::delete(schema_versions).execute(conn)
        })
        .unwrap();
    }

    #[allow(dead_code)]
    fn reset_schemas(&mut self) {
        use avro_schema_registry::db::models::schema::schemas::dsl::*;
        diesel::delete(schemas).execute(self).unwrap();
    }

    fn reset_subjects(&mut self) {
        use avro_schema_registry::db::models::schema::subjects::dsl::*;
        diesel::delete(subjects).execute(self).unwrap();
    }

    #[allow(dead_code)]
    fn reset_configs_global(&mut self) {
        use avro_schema_registry::db::models::schema::configs::dsl::*;

        self.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::update(configs)
                .filter(id.eq(0))
                .set(compatibility.eq("BACKWARD"))
                .execute(conn)
        })
        .unwrap();
    }

    fn create_test_subject_with_config(&mut self, compat: &str) {
        use avro_schema_registry::db::models::schema::configs::dsl::{
            compatibility, configs, created_at as config_created_at, subject_id,
            updated_at as config_updated_at,
        };
        use avro_schema_registry::db::models::schema::subjects::dsl::*;
        use avro_schema_registry::db::models::Subject;

        self.transaction::<_, diesel::result::Error, _>(|conn| {
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

    fn add_subjects(&mut self, subjects: Vec<String>) {
        use avro_schema_registry::api::errors::ApiError;
        use avro_schema_registry::db::models::Subject;

        self.transaction::<String, ApiError, _>(|conn| {
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

    fn register_schema(&mut self, subject: String, schema: String) -> Schema {
        use avro_schema_registry::db::models::RegisterSchema;
        Schema::register_new_version(self, RegisterSchema { subject, schema }).unwrap()
    }
}

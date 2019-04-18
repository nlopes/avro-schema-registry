pub fn reset(conn: &super::PgConnection) {
    use avro_schema_registry::db::models::schema::schemas::dsl::*;
    use diesel::prelude::*;
    diesel::delete(schemas).execute(conn).unwrap();
}

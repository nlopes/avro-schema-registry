pub fn reset(conn: &super::PgConnection) {
    use diesel::prelude::*;

    use avro_schema_registry::db::models::schema::configs::dsl::configs;
    use avro_schema_registry::db::models::schema::schema_versions::dsl::schema_versions;
    use avro_schema_registry::db::models::schema::schemas::dsl::schemas;
    use avro_schema_registry::db::models::schema::subjects::dsl::subjects;

    conn.transaction::<_, diesel::result::Error, _>(|| {
        diesel::delete(configs).execute(conn)?;
        diesel::delete(schemas).execute(conn)?;
        diesel::delete(subjects).execute(conn)?;
        diesel::delete(schema_versions).execute(conn)
    })
    .unwrap();
}

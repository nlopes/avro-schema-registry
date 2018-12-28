use diesel::prelude::*;

pub fn reset_global(conn: &super::PgConnection) {
    use avro_schema_registry::db::models::schema::configs::dsl::*;

    conn.transaction::<_, diesel::result::Error, _>(|| {
        diesel::update(configs)
            .filter(id.eq(0))
            .set(compatibility.eq("BACKWARD"))
            .execute(conn)
    })
    .unwrap();
}

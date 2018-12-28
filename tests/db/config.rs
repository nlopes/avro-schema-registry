use diesel::prelude::*;

use super::connection;

pub fn reset_global() {
    use avro_schema_registry::db::models::schema::configs::dsl::*;
    let conn = connection::connection();

    conn.transaction::<_, diesel::result::Error, _>(|| {
        diesel::update(configs)
            .filter(id.eq(0))
            .set(compatibility.eq("BACKWARD"))
            .execute(&conn)
    })
    .unwrap();
}

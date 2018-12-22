use super::schema::*;
use super::schemas::Schema;
use super::subjects::Subject;

use diesel::prelude::*;

#[derive(Debug, Identifiable, Associations, Queryable)]
#[table_name = "schema_versions"]
#[belongs_to(Schema)]
#[belongs_to(Subject)]
pub struct SchemaVersion {
    pub id: i64,
    pub version: Option<i32>,
    pub subject_id: i64,
    pub schema_id: i64,
}

impl SchemaVersion {
    // TODO: I'm not happy with this. Positional arguments are so prone to errors it's not
    // even funny. Part of the issue is that we call this function usually after a select,
    // and said select only allows doing this with tuples. Sucks.
    pub fn from_tuple((id, version, subject_id, schema_id): (i64, Option<i32>, i64, i64)) -> Self {
        SchemaVersion {
            id: id,
            version: version,
            subject_id: subject_id,
            schema_id: schema_id,
        }
    }

    pub fn delete_subject_with_name(
        subject: String,
        conn: &PgConnection,
    ) -> Result<Vec<Option<i32>>, diesel::result::Error> {
        use super::schema::schema_versions::dsl::{
            id, schema_id, schema_versions, subject_id, version,
        };
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, name, subjects};

        conn.transaction::<_, diesel::result::Error, _>(|| {
            Ok(schema_versions
                .inner_join(subjects.on(subject_id.eq(subjects_id)))
                .inner_join(schemas.on(schema_id.eq(schemas_id)))
                .filter(name.eq(&subject))
                .select((id, version, subject_id, schema_id))
                .load::<(i64, Option<i32>, i64, i64)>(conn)?
                .into_iter()
                .map(|entry| {
                    let sv = SchemaVersion::from_tuple(entry);
                    match sv.delete(&conn) {
                        Err(e) => {
                            info!("error deleting: {}", e);
                        }
                        _ => (),
                    };
                    sv.version
                })
                .collect())
        })
    }

    fn delete(&self, conn: &PgConnection) -> Result<(), diesel::result::Error> {
        use super::schema::schema_versions::dsl::{id, schema_versions};
        use super::schema::schemas::dsl::{id as schemas_id, schemas};
        use super::schema::subjects::dsl::{id as subjects_id, subjects};

        conn.transaction::<_, diesel::result::Error, _>(|| {
            let schemas_delete = schemas.filter(schemas_id.eq(self.schema_id));
            let subjects_delete = subjects.filter(subjects_id.eq(self.subject_id));
            let schema_versions_delete = schema_versions.filter(id.eq(self.id));

            diesel::delete(schemas_delete).execute(conn)?;
            diesel::delete(subjects_delete).execute(conn)?;
            diesel::delete(schema_versions_delete).execute(conn)?;

            Ok(())
        })
    }
}

use std::env;

use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

use std::env;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use crate::api::errors::{ApiAvroErrorCode, ApiError};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub trait DbManage {
    fn new_pool(max_size: Option<u32>) -> Self;
    fn connection(&self) -> Result<DbConnection, ApiError>;
}

impl DbManage for DbPool {
    fn new_pool(max_size: Option<u32>) -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        Pool::builder()
            .max_size(max_size.unwrap_or(10))
            .build(manager)
            .expect("Failed to create pool.")
    }

    fn connection(&self) -> Result<DbConnection, ApiError> {
        self.get()
            .map_err(|_| ApiError::new(ApiAvroErrorCode::BackendDatastoreError))
    }
}

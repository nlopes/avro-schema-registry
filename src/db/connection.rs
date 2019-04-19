use std::env;

use actix::{Actor, Addr, SyncArbiter, SyncContext};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use crate::api::errors::{ApiAvroErrorCode, ApiError};

pub struct ConnectionPooler(pub Pool<ConnectionManager<PgConnection>>);
pub type PoolerAddr = Addr<ConnectionPooler>;

impl Actor for ConnectionPooler {
    type Context = SyncContext<Self>;
}

type DBConnection = PooledConnection<ConnectionManager<PgConnection>>;

impl ConnectionPooler {
    pub fn pool() -> Pool<ConnectionManager<PgConnection>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        Pool::new(manager).expect("Failed to create pool.")
    }

    pub fn init(n_workers: usize) -> Addr<Self> {
        let pool = ConnectionPooler::pool();
        SyncArbiter::start(n_workers, move || ConnectionPooler(pool.clone()))
    }

    pub fn connection(&self) -> Result<DBConnection, ApiError> {
        self.0
            .get()
            .map_err(|_| ApiError::new(ApiAvroErrorCode::BackendDatastoreError))
    }
}

use std::env;

use actix_web::actix::{Actor, Addr, SyncArbiter, SyncContext};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use crate::api::errors::{ApiError, ApiErrorCode, ApiStatusCode};

pub struct ConnectionPooler(pub Pool<ConnectionManager<PgConnection>>);
impl Actor for ConnectionPooler {
    type Context = SyncContext<Self>;
}

type DBConnection = PooledConnection<ConnectionManager<PgConnection>>;

impl ConnectionPooler {
    pub fn init() -> Addr<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let conn = Pool::new(manager).expect("Failed to create pool.");
        // TODO: remove this magic number 4
        SyncArbiter::start(4, move || ConnectionPooler(conn.clone()))
    }

    pub fn connection(&self) -> Result<DBConnection, ApiError> {
        self.0.get().map_err(|e| {
            ApiError::with_message(
                ApiStatusCode::InternalServerError,
                ApiErrorCode::BackendDatastoreError,
                format!("{}", e),
            )
        })
    }
}

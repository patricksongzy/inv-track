use std::env;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// The default number of max connections.
const DEFAULT_MAX_CONNECTIONS: u32 = 100;

/// Gets the database connection pool.
pub(crate) async fn get_pool() -> Pool<Postgres> {
    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .map(|val| val.parse::<u32>().unwrap_or(DEFAULT_MAX_CONNECTIONS))
        .unwrap_or(DEFAULT_MAX_CONNECTIONS);

    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("unable to establish database pool")
}

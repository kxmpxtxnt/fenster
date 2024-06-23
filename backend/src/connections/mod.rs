use anyhow::Result;
use log::{Level, log};
use redis::Client;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

use crate::config::{Host, Postgres};
use crate::handlers::FensterError;
use crate::handlers::FensterError::Internal;

pub async fn create_postgres_pool(postgres: Postgres) -> Result<PgPool, FensterError> {
    match PgPool::connect(format!("postgresql://{}:{}@{}:{}/fenster", postgres.user, postgres.password, postgres.address, postgres.port).as_str()).await {
        Ok(pool) => Ok(pool),
        Err(err) => {
            log!(Level::Error, "{}", err);
            Err(Internal)
        }
    }
}

pub async fn create_redis_connection(redis: Host) -> Result<MultiplexedConnection, FensterError> {
    match Client::open(
        format!("redis://{}:{}", redis.address, redis.port)
    ) {
        Ok(client) => Ok(client.get_multiplexed_async_connection().await.unwrap()),
        Err(err) => {
            log!(Level::Error, "{}", err);
            Err(Internal)
        }
    }
}
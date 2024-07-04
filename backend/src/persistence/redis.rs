use std::process::exit;

use tracing::error;
use redis::aio::MultiplexedConnection;
use redis::Client;

use crate::backend_config::Host;
use crate::fenster_error::FensterError;

pub async fn create_redis_connection(redis: Host) -> anyhow::Result<MultiplexedConnection, FensterError> {
    let connection = Client::open(format!("redis://{}:{}", redis.address, redis.port))
        .inspect_err(|err| {
            error!("Unable to connect to redis. - {}", err)
        })
        .map_err(|_| exit(0))?;

    Ok(connection.get_multiplexed_async_connection().await.unwrap())
}
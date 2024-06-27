use std::process::exit;

use anyhow::Ok;
use anyhow::Result;
use log::error;
use sqlx::{PgPool, Pool, Postgres};

pub async fn create_postgres_pool(postgres: crate::backend_config::Postgres) -> Result<Pool<Postgres>> {
    let pool = PgPool::connect(format!("postgresql://{}:{}@{}:{}/fenster", postgres.user, postgres.password, postgres.address, postgres.port).as_str()).await
        .inspect_err(|err| {
            error!("Unable to connect to database. - {}", err)
        })
        .map_err(|_| exit(0)).unwrap();

    Ok(pool)
}
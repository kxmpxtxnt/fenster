use std::fs;
use std::time::Duration;

use anyhow::Result;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::Router;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tracing::info;
use crate::backend_config::BackendConfig;

pub(crate) mod persistence;
pub(crate) mod auth;
pub(crate) mod user;
pub(crate) mod article;
pub(crate) mod fenster_error;
pub(crate) mod backend_config;

#[derive(Clone)]
pub struct AppInject {
    pub postgres_pool: PgPool,
    pub redis_connection: MultiplexedConnection,
    pub backend_config: BackendConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt().finish()
    )?;
    
    let backend_config: BackendConfig = toml::from_str(fs::read_to_string("./config.toml")
        .unwrap().as_str())
        .unwrap();

    let postgres_pool = persistence::postgres::create_postgres_pool(backend_config.clone().postgres).await
        .expect("configuration postgres should lead to a postgres server.");

    let redis_connection = persistence::redis::create_redis_connection(backend_config.clone().redis).await
        .expect("configuration redis should lead to a redis server.");

    let inject = AppInject {
        postgres_pool,
        redis_connection,
        backend_config: backend_config.clone(),
    };

    let router = Router::new()
        .nest("/user", user::user_handler::user_router())
        .nest("/auth", auth::auth_handler::auth_router())
        .nest("/article", article::article_handler::article_router())
        .route_layer(ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|err| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled error: {}", err),
                )
            }))
            .layer(BufferLayer::new(1024))
            .layer(RateLimitLayer::new(5, Duration::from_secs(1)))
        )
        .with_state(inject);

    let host = backend_config.clone().host;

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", host.address, host.port)
    ).await?;

    info!("Listening on {}:{}", host.address, host.port);
    axum::serve(listener, router).await?;
    Ok(())
}
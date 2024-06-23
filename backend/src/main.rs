use std::fs;
use std::time::Duration;

use anyhow::Result;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::Router;
use axum::routing::{get, post, put};
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;

use crate::config::BackendConfig;
use crate::handlers::{articles, auth, root, users};

pub mod entities;
pub mod handlers;
pub mod connections;
mod authentication;
mod config;

#[derive(Clone)]
pub struct AppInject {
    pub postgres_pool: PgPool,
    pub redis_connection: MultiplexedConnection,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config: BackendConfig = toml::from_str(fs::read_to_string("./config.toml")
        .unwrap().as_str())
        .unwrap();

    let postgres_pool = connections::create_postgres_pool(config.postgres).await
        .expect("env DATABASE_URL should lead to a postgres server.");

    let redis_connection = connections::create_redis_connection(config.redis).await
        .expect("env REDIS_URL should lead to a redis server.");

    let inject = AppInject {
        postgres_pool,
        redis_connection,
    };

    let router = Router::new()
        .route("/", get(root::root))
        .route("/u/login", put(auth::login))
        .route("/u/:id", get(users::get_user))
        .route("/u/register", post(auth::register))
        .route("/a/", post(articles::create_article))
        .route("/a/:slug", get(articles::get_article))
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

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.host.address, config.host.port)
    ).await?;

    println!("Listening on {}:{}", config.host.address, config.host.port);
    axum::serve(listener, router).await?;
    Ok(())
}
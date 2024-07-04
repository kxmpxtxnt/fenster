use axum_extra::headers::authorization::Bearer;
use redis::aio::MultiplexedConnection;
use serde::Deserialize;

use crate::fenster_error::FensterError;

pub(crate) mod token_entity;
pub(crate) mod auth_handler;

#[derive(Deserialize)]
pub struct RegisterUser {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub(crate) id: String,
    pub(crate) password: String,
}

pub async fn require_authentication(bearer: Bearer, redis: MultiplexedConnection) -> Result<(), FensterError> {
    let access_token = String::from(bearer.token());
    
    let user_id = token_entity::user_id_from_token(access_token, redis.clone()).await?;

    let token = token_entity::token_from_user_id(user_id, redis.clone()).await?;

    token.auth_token.is_expired()
}
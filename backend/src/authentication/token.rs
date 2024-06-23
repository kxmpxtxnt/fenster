use std::iter;
use std::string::String;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use log::{Level, log};
use rand::Rng;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::entities::user::User;
use crate::handlers::FensterError;
use crate::handlers::FensterError::{Internal, Unauthorized};

const CHARS: &str =
    "1234567890!§$%&/()=abcdefghijklmopqrstuvwxyzABCDEFGHIJKLMOPQRSTUVWXYZ_-.,:;#'*<>?ß}][{";

#[derive(Clone, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
pub struct Token {
    auth_token: AccessToken,
    refresh_token: AccessToken,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AccessToken {
    token: String,
    expiration_period: u64,
}

pub async fn create_token(user: User, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let token = Token {
        auth_token: token_with_expiration(7)?,
        refresh_token: token_with_expiration(14)?,
    };

    let token_json = serde_json::to_string(&token).map_err(|err| {
        log!(Level::Error, "{}", err);
        Internal
    })?;

    redis.set(&user.id, &token_json).await.map_err(|err| {
        log!(Level::Error, "{}", err);
        Internal
    })?;

    redis.set(&token.auth_token.token, &user.id).await.map_err(|err| {
        log!(Level::Error, "{}", err);
        Internal
    })?;

    redis.set(&token.refresh_token.token, &user.id).await.map_err(|err| {
        log!(Level::Error, "{}", err);
        Internal
    })?;

    Ok(token)
}

pub async fn find_token(access_token: &String, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let user_id: Result<String, FensterError> = redis.get(access_token).await.map_err(|err| {
        log!(Level::Error, "{}", err);
        Unauthorized
    });

    match redis.get(&user_id?).await {
        Ok(token) => Ok(token),
        Err(err) => {
            log!(Level::Error, "{}", err);
            Err(Internal)
        }
    }
}

pub async fn refresh_access(refresh_token: String, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let user_id = redis.get(refresh_token).await.map_err(|_| Unauthorized)?;

    let token = match find_token(&user_id, redis.clone()).await {
        Ok(token) => Ok(token),
        Err(_) => Err(Unauthorized)
    }.map_err(|_| Unauthorized)?;

    redis.del(&token.auth_token.token).await.map_err(|_| Internal)?;

    let new_auth_token = token_with_expiration(7)?;

    redis.set(&new_auth_token.token, &user_id).await.map_err(|_| Internal)?;

    Ok(Token {
        auth_token: new_auth_token,
        refresh_token: token.refresh_token,
    })
}

pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    iter::repeat_with(
        || CHARS.as_bytes()[rng.gen_range(0..CHARS.len())] as char
    ).take(16).collect()
}

fn token_with_expiration(days: u64) -> Result<AccessToken, FensterError> {
    let sys_time = SystemTime::now().duration_since(UNIX_EPOCH);

    Ok(AccessToken {
        token: generate_token(),
        expiration_period: (sys_time.map_err(|_| Internal).unwrap().as_millis()
            + Duration::from_secs(60 * 60 * 24 * days).as_millis()) as u64,
    })
}
use std::iter;
use std::string::String;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use log::{error};
use rand::Rng;
use redis::{AsyncCommands};
use redis::aio::MultiplexedConnection;
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::fenster_error::{error, FensterError, REDIS_ERROR, SERDE_ERROR, OTHER_INTERNAL_ERROR};
use crate::fenster_error::FensterError::{Internal};
use crate::user::user_entity::User;

const CHARS: &str =
    "1234567890!§$%&/()=abcdefghijklmopqrstuvwxyzABCDEFGHIJKLMOPQRSTUVWXYZ_-.,:;#'*<>?ß}][{";

#[derive(Clone, Debug, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
pub struct Token {
    auth_token: AccessToken,
    refresh_token: AccessToken,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessToken {
    token: String,
    expiration_period: u128,
}

pub async fn create_token(user: User, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let token = Token {
        auth_token: token_with_expiration(7)?,
        refresh_token: token_with_expiration(14)?,
    };

    let token_json = serde_json::to_string(&token)
        .inspect_err(|err| {
            error!("Unable to serialize token to json. - {}", err)
        })
        .map_err(|_| Internal(error(SERDE_ERROR, 1)))?;

    redis.set(&user.id, &token_json).await
        .inspect_err(|err| {
            error!("Unable to set user_id ({}) to token. - {}",user.id, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 1)))?;

    redis.set(&token.auth_token.token, &user.id).await
        .inspect_err(|err| {
            error!("Unable to set user_id ({}) to auth_token. - {}", user.id, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 2)))?;

    redis.set(&token.refresh_token.token, &user.id).await
        .inspect_err(|err| {
            error!("Unable to set user_id ({}) to refresh_token. - {}", user.id, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 3)))?;

    Ok(token)
}

pub async fn find_token(token: &String, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let user_id: String = redis.get(token).await
        .inspect_err(|err| {
            error!("Unable to get user_id from token ({token}) - {}", err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 4)))?;

    let token = redis.get(&user_id).await
        .inspect_err(|err| {
            error!("Unable to get token from user_id ({}) - {}", &user_id, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 5)))?;

    Ok(token)
}

pub async fn refresh_access(refresh_token: String, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let user_id = redis.get(&refresh_token).await
        .inspect_err(|err| {
            error!("Unable to get user_id from refresh_token ({}) - {}", &refresh_token, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 6)))?;


    let token = find_token(&user_id, redis.clone()).await?;

    redis.del(&token.auth_token.token).await
        .inspect_err(|err| {
            error!("Unable to delete user_id from auth_token ({}) - {}", token.auth_token.token, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 7)))?;

    let access_token = token_with_expiration(7)?;

    redis.set(&access_token.token, &user_id).await
        .inspect_err(|err| {
            error!("Unable to set user_id ({}) to auth_token ({}) - {}", user_id, token.auth_token.token, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 8)))?;

    Ok(Token {
        auth_token: access_token,
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
    let sys_time = SystemTime::now().duration_since(UNIX_EPOCH)
        .inspect_err(|err| {
            error!("Error while loading duration since unix_epoch. - {}", err)
        })
        .map_err(|_| Internal(error(OTHER_INTERNAL_ERROR, 2)))?;

    Ok(AccessToken {
        token: generate_token(),
        expiration_period: sys_time.as_millis()
            + Duration::from_secs(60 * 60 * 24 * days).as_millis(),
    })
}
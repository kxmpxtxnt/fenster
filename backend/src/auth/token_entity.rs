use std::iter;
use std::string::String;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use rand::Rng;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::fenster_error::{error, FensterError, OTHER_INTERNAL_ERROR, REDIS_ERROR};
use crate::fenster_error::FensterError::{Internal, Unauthorized};
use crate::user::user_entity::User;

const CHARS: &str =
    "1234567890abcdefghijklmopqrstuvwxyzABCDEFGHIJKLMOPQRSTUVWXYZ";

#[derive(Clone, Debug, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
pub struct Token {
    pub auth_token: AccessToken,
    pub refresh_token: AccessToken,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRedisValue, ToRedisArgs)]
pub struct AccessToken {
    pub token: String,
    pub expiration_period: u128,
}

impl AccessToken {
    pub fn is_expired(&self) -> Result<(), FensterError> {
        let sys_time = get_sys_time()?;

        if sys_time.as_millis() >= self.expiration_period {
            return Err(Unauthorized(format!("{} token is expired.", self.token)));
        }

        Ok(())
    }
}

pub async fn create_token(user: User, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let token = Token {
        auth_token: token_with_expiration(7)?,
        refresh_token: token_with_expiration(14)?,
    };

    redis.set(user.clone().id, token.clone()).await
        .inspect_err(|err| {
            error!("Unable to set token to user_id ({}). - {}", user.clone().id, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 1)))?;
    
    redis.mset(&[
        (token.clone().auth_token.token, user.clone().id),
        (token.clone().refresh_token.token, user.clone().id)
    ]).await
        .inspect_err(|err| {
            error!("Unable to set auth/refresh _token to user_id ({}). - {}", user.clone().id, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 2)))?;

    Ok(token)
}

pub async fn user_id_from_token(token: String, mut redis: MultiplexedConnection) -> Result<String, FensterError> {
    let user_id = redis.get(token.clone()).await
        .inspect_err(|err| {
            error!("Unable to get user_id from token ({token}) - {}", err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 3)))?;

    Ok(user_id)
}

pub async fn token_from_user_id(user_id: String, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let token: Token = redis.get(user_id.clone()).await
        .inspect_err(|err| {
            error!("Unable to get token from user_id ({}) - {}", user_id.clone(), err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 5)))?;

    Ok(token)
}

pub async fn refresh_access(token: String, mut redis: MultiplexedConnection) -> Result<Token, FensterError> {
    let user_id = user_id_from_token(token, redis.clone()).await?;

    let token = token_from_user_id(user_id.clone(), redis.clone()).await?;

    redis.del(token.clone().auth_token.token).await
        .inspect_err(|err| {
            error!("Unable to delete user_id from auth_token ({}) - {}", token.clone().auth_token.token, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 6)))?;

    let access_token = token_with_expiration(7)?;

    redis.set(access_token.clone().token, user_id.clone()).await
        .inspect_err(|err| {
            error!("Unable to set user_id ({}) to auth_token ({}) - {}", user_id, token.clone().auth_token.token, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 7)))?;

    Ok(Token {
        auth_token: access_token.clone(),
        refresh_token: token.refresh_token,
    })
}

pub async fn revoke_access(access_token: String, mut redis: MultiplexedConnection) -> Result<(), FensterError> {
    
    let user_id = user_id_from_token(access_token.clone(), redis.clone()).await?;

    let token = token_from_user_id(user_id.clone(), redis.clone()).await?;

    redis.del(token.clone().auth_token.token).await
        .inspect_err(|err| {
            error!("Unable to delete user_id from auth_token ({}) - {}", token.clone().auth_token.token, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 8)))?;

    redis.del(token.clone().refresh_token.token).await
        .inspect_err(|err| {
            error!("Unable to set user_id from refresh_token ({}) - {}", token.clone().auth_token.token, err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 9)))?;

    redis.del(user_id.clone()).await
        .inspect_err(|err| {
            error!("Unable to delete access_token from user_id ({}) - {}", user_id.clone(), err)
        })
        .map_err(|_| Internal(error(REDIS_ERROR, 10)))?;

    Ok(())
}

fn token_with_expiration(days: u64) -> Result<AccessToken, FensterError> {
    let sys_time = get_sys_time()?;

    Ok(AccessToken {
        token: generate_token(),
        expiration_period: sys_time.as_millis()
            + Duration::from_secs(60 * 60 * 24 * days).as_millis(),
    })
}

fn get_sys_time() -> Result<Duration, FensterError> {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .inspect_err(|err| {
            error!("Error while loading duration since unix_epoch. - {}", err)
        })
        .map_err(|_| Internal(error(OTHER_INTERNAL_ERROR, 2)))
}


pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    iter::repeat_with(
        || CHARS.as_bytes()[rng.gen_range(0..CHARS.len())] as char
    ).take(16).collect()
}
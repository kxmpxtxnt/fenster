use anyhow::Result;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{PasswordHashString, SaltString};
use argon2::password_hash::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;

use crate::fenster_error::{error, FensterError, POSTGRES_ERROR};
use crate::fenster_error::FensterError::{Internal, NotFound};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) author: bool,
}

impl User {
    pub async fn store(&self, password: &str, pool: &PgPool) -> Result<bool, FensterError> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash_string = argon.hash_password(password.as_ref(), &salt)
            .inspect_err(|err| {
                error!("Error while creating password hash string. - {}", err);
            })
            .map_err(|_| Internal(error(POSTGRES_ERROR, 4)))?
            .serialize();

        let result = sqlx::query!(
            "INSERT INTO fenster.public.users(user_id, user_name, user_email, user_password_hash)
            VALUES($1, $2, $3, $4)",
            &self.id, &self.name, &self.email, hash_string.as_str())
            .execute(pool)
            .await
            .inspect_err(|err| {
                error!("Error while storing user with id ({}). - {}", self.id, err);
            })
            .map_err(|_| Internal(error(POSTGRES_ERROR, 4)))?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn matches(&self, password: &str, pool: &PgPool) -> Result<bool, FensterError> {
        let hash = sqlx::query!(
            "SELECT user_password_hash FROM fenster.public.users WHERE user_id=$1", &self.id)
            .fetch_one(pool)
            .await
            .inspect_err(|err| {
                error!("Error while finding user_password_hash for user with id ({}). - {}", self.id, err)
            })
            .map_err(|_| Internal(error(POSTGRES_ERROR, 5)))?;

        let result = Argon2::default()
            .verify_password(password.as_ref(),
                             &PasswordHashString::new(hash.user_password_hash.as_str())
                                 .inspect_err(|err| {
                                     error!("Error while verifying password hash for user with id ({}). - {}", self.id, err);
                                 })
                                 .map_err(|_| Internal(error(POSTGRES_ERROR, 6)))?
                                 .password_hash(),
            );

        Ok(result.is_ok())
    }
}

pub async fn exists_id(id: &str, pool: &PgPool) -> Result<bool, FensterError> {
    let result = sqlx::query!(
        "SELECT EXISTS (SELECT user_name FROM fenster.public.users WHERE user_id=$1)", &id)
        .fetch_one(pool)
        .await
        .inspect_err(|err| {
            error!("Error while finding user by id ({}). - {}", id, err);
        })
        .map_err(|_| {
            Internal(error(POSTGRES_ERROR, 7))
        })?;

    Ok(result.exists.unwrap_or(false))
}

pub async fn exists_mail(mail: &str, pool: &PgPool) -> Result<bool, FensterError> {
    let result = sqlx::query!(
        "SELECT EXISTS (SELECT user_name FROM fenster.public.users WHERE user_email=$1)", &mail)
        .fetch_one(pool)
        .await
        .inspect_err(|err| {
            error!("Error while findung user by email ({}). - {}", mail, err);
        })
        .map_err(|_| {
            Internal(error(POSTGRES_ERROR, 8))
        })?;

    Ok(result.exists.unwrap_or(false))
}

pub async fn fetch(id: &str, pool: &PgPool) -> Result<User, FensterError> {
    if !exists_id(id, pool).await.unwrap_or(false) {
        return Err(NotFound("User does not exist.".to_string()));
    }

    let user = sqlx::query!(
        "SELECT user_id, user_name, user_email, user_author FROM fenster.public.users WHERE user_id=$1", &id)
        .fetch_one(pool)
        .await
        .inspect_err(|err| {
            error!("Error while fetching user with id ({}). - {}", id, err);
        })
        .map_err(|_| {
            Internal(error(POSTGRES_ERROR, 9))
        })?;

    Ok(User {
        id: user.user_id,
        name: user.user_name,
        email: user.user_email,
        author: user.user_author.unwrap_or(false),
    })
}

pub async fn delete(id: &str, pool: &PgPool) -> Result<(), FensterError> {
    if !exists_id(id, pool).await.unwrap_or(false) {
        return Err(NotFound("User does not exist.".to_string()));
    }

    sqlx::query!("DELETE FROM fenster.public.users WHERE user_id=$1", id)
        .execute(pool)
        .await
        .inspect_err(|err| {
            error!("Error while delete user with id ({}). - {}", id, err);
        })
        .map_err(|_| {
            Internal(error(POSTGRES_ERROR, 10))
        })?;

    Ok(())
}
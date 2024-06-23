use anyhow::Result;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{PasswordHashString, SaltString};
use argon2::password_hash::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::handlers::FensterError;
use crate::handlers::FensterError::Internal;

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
        let hash_string = argon.hash_password(password.as_ref(), &salt).map_err(|_| Internal).unwrap().serialize();

        let result = sqlx::query!(
            "INSERT INTO fenster.public.users(user_id, user_name, user_email, user_password_hash)
            VALUES($1, $2, $3, $4)",
            &self.id, &self.name, &self.email, hash_string.as_str())
            .execute(pool)
            .await;

        if !result.is_ok() {
            return Err(Internal);
        }

        Ok(result.unwrap().rows_affected() != 0)
    }

    pub async fn matches(&self, password: &str, pool: &PgPool) -> Result<bool, FensterError> {
        let hash = sqlx::query!(
            "SELECT user_password_hash FROM fenster.public.users WHERE user_id=$1", &self.id)
            .fetch_one(pool)
            .await.map_err(|_| Internal)?;

        let result = Argon2::default()
            .verify_password(password.as_ref(), &PasswordHashString::new(hash.user_password_hash.as_str()).map_err(|_| Internal)?.password_hash()).is_ok();

        Ok(result)
    }
}

pub async fn exists(id: &str, pool: &PgPool) -> Result<bool, FensterError> {
    let result = sqlx::query!(
        "SELECT EXISTS (SELECT user_name FROM fenster.public.users WHERE user_id=$1)", &id)
        .fetch_one(pool)
        .await.map_err(|_| Internal)?;

    Ok(result.exists.unwrap_or(false))
}

pub async fn fetch(id: &str, pool: &PgPool) -> Result<User, FensterError> {
    let user = sqlx::query!(
        "SELECT user_id, user_name, user_email, user_author FROM fenster.public.users WHERE user_id=$1", &id)
        .fetch_one(pool)
        .await.map_err(|_| Internal)?;

    Ok(User {
        id: user.user_id,
        name: user.user_name,
        email: user.user_email,
        author: user.user_author.unwrap_or(false),
    })
}
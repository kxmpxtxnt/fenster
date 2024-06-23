use axum::extract::State;
use axum::Json;
use axum::response::Redirect;
use serde::Deserialize;
use crate::AppInject;

use crate::authentication::token;
use crate::entities::user;
use crate::entities::user::User;
use crate::handlers::FensterError;
use crate::handlers::FensterError::Internal;

#[derive(Deserialize)]
pub struct LoginUser {
    pub(crate) id: String,
    pub(crate) password: String,
}

pub async fn login(
    State(AppInject { postgres_pool, redis_connection }): State<AppInject>,
    Json(login): Json<LoginUser>,
) -> Result<Json<token::Token>, FensterError> {
    let exists = user::exists(login.id.as_str(), &postgres_pool).await.unwrap_or(false);

    if !exists {
        return Err(FensterError::Conflict);
    }

    let user = user::fetch(login.id.as_str(), &postgres_pool).await.map_err(|_| Internal)?;

    if !user.matches(login.password.as_str(), &postgres_pool).await? {
        return Err(FensterError::Unauthorized);
    }

    match token::create_token(user, redis_connection).await {
        Ok(token) => Ok(Json(token)),
        Err(err) => Err(err)
    }
}

#[derive(Deserialize)]
pub struct RegisterUser {
    id: String,
    name: String,
    email: String,
    password: String,
}

pub async fn register(
    State(AppInject { postgres_pool, .. }): State<AppInject>,
    Json(register): Json<RegisterUser>,
) -> Result<Redirect, FensterError> {
    let exists = user::exists(register.id.as_str(), &postgres_pool).await.unwrap_or(true);

    if exists {
        return Err(FensterError::Conflict);
    }

    let user = User {
        id: register.id,
        name: register.name,
        email: register.email,
        author: false,
    };

    match user.store(register.password.as_str(), &postgres_pool).await.unwrap_or(false) {
        true => Ok(Redirect::to("/u/login")),
        false => Err(Internal),
    }
}
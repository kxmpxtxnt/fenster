use anyhow::Result;
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{post, put};
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use tracing::error;

use crate::{AppInject, user::user_entity};
use crate::auth::{LoginUser, RefreshBody, RegisterUser, require_authentication, token_entity};
use crate::auth::token_entity::Token;
use crate::fenster_error::{error, FensterError, OTHER_INTERNAL_ERROR};
use crate::fenster_error::FensterError::{Conflict, Internal, Unauthorized};
use crate::user::user_entity::User;

pub fn auth_router() -> Router<AppInject> {
    Router::new()
        .route("/login", put(login))
        .route("/logout", put(logout))
        .route("/refresh", put(refresh))
        .route("/register", post(register))
        .route("/delete", put(delete))
}

pub async fn login(
    State(AppInject { postgres_pool, redis_connection, .. }): State<AppInject>,
    Json(login): Json<LoginUser>,
) -> Result<Json<Token>, FensterError> {
    let user = user_entity::fetch(login.id.as_str(), &postgres_pool).await?;

    if !user.matches(login.password.as_str(), &postgres_pool).await? {
        return Err(Unauthorized(format!("Password for user with given id ({}) is incorrect.", login.id)));
    }

    let token = token_entity::create_token(user, redis_connection).await?;
    Ok(Json(token))
}

pub async fn logout(
    State(AppInject { redis_connection, .. }): State<AppInject>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, FensterError> {
    require_authentication(bearer.clone(), redis_connection.clone()).await?;

    token_entity::revoke_access(bearer.token().to_string(), redis_connection.clone()).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn register(
    State(AppInject { postgres_pool, backend_config, .. }): State<AppInject>,
    Json(register): Json<RegisterUser>,
) -> Result<StatusCode, FensterError> {
    let school = backend_config.school.clone();

    let regex = regex::Regex::new(school.mail_pattern.as_str())
        .inspect_err(|err| {
            error!("Error parsing school mail regex pattern. ({}) - {}", school.mail_pattern, err)
        })
        .map_err(|_| Internal(error(OTHER_INTERNAL_ERROR, 1)))?;

    if !regex.is_match(register.email.as_str()) {
        return Err(Conflict(format!("Mail ({}) does not match the pattern (Example {}).", register.email, school.example_mail).to_string()));
    }

    if user_entity::exists_id(register.id.as_str(), &postgres_pool).await? {
        return Err(Conflict(format!("User with given id ({}) already exists.", register.id)));
    }

    if user_entity::exists_mail(register.email.as_str(), &postgres_pool).await? {
        return Err(Conflict(format!("User with given email ({}) already exists.", register.email)));
    }

    let user = User {
        id: register.id,
        name: register.name,
        email: register.email,
        author: false,
    };

    user.store(register.password.as_str(), &postgres_pool).await?;
    Ok(StatusCode::CREATED)
}

pub async fn refresh(
    State(AppInject { redis_connection, .. }): State<AppInject>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(refresh): Json<RefreshBody>,
) -> Result<Json<Token>, FensterError> {
    require_authentication(bearer.clone(), redis_connection.clone()).await?;
    let token = token_entity::refresh_access(refresh.refresh_token, redis_connection.clone()).await?;
    Ok(Json(token))
}

pub async fn delete(
    State(AppInject { redis_connection, postgres_pool, .. }): State<AppInject>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>
) -> Result<StatusCode, FensterError> {
    let user_id = require_authentication(bearer.clone(), redis_connection.clone()).await?;
    token_entity::revoke_access(bearer.token().to_string(), redis_connection).await?;
    user_entity::delete(user_id.as_str(), &postgres_pool).await?;
    Ok(StatusCode::OK)
}
use axum::extract::State;
use axum::Json;
use axum::response::Redirect;
use log::error;

use crate::{AppInject, user::user_entity};
use crate::auth::{LoginUser, RegisterUser, token_entity};
use crate::fenster_error::{error, FensterError, OTHER_INTERNAL_ERROR};
use crate::fenster_error::FensterError::{Conflict, Internal, Unauthorized};
use crate::user::user_entity::User;

pub async fn login(
    State(AppInject { postgres_pool, redis_connection, .. }): State<AppInject>,
    Json(login): Json<LoginUser>,
) -> Result<Json<token_entity::Token>, FensterError> {
    let user = user_entity::fetch(login.id.as_str(), &postgres_pool).await?;

    if !user.matches(login.password.as_str(), &postgres_pool).await? {
        return Err(Unauthorized(format!("Password for user with given id ({}) is incorrect.", login.id)));
    }

    match token_entity::create_token(user, redis_connection).await {
        Ok(token) => Ok(Json(token)),
        Err(err) => Err(err)
    }
}

pub async fn register(
    State(AppInject { postgres_pool, backend_config, .. }): State<AppInject>,
    Json(register): Json<RegisterUser>,
) -> Result<Redirect, FensterError> {
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
    Ok(Redirect::to("/u/login"))
}
use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::routing::get;

use crate::{AppInject, user};
use crate::fenster_error::FensterError;
use crate::user::user_entity::User;

use anyhow::Result;

pub fn user_router() -> Router<AppInject> {
    Router::new()
        .route("/:id", get(get_user))
}

pub async fn get_user(
    State(AppInject { postgres_pool, .. }): State<AppInject>,
    Path(id): Path<String>,
) -> Result<Json<User>, FensterError> {
    let exists = user::user_entity::exists_id(&id, &postgres_pool).await?;

    if !exists {
        return Err(FensterError::NotFound("User does not exist.".to_string()));
    }

    let user = user::user_entity::fetch(&id, &postgres_pool).await?;
    Ok(Json(user))
}
use axum::extract::{Path, State};
use axum::Json;
use crate::AppInject;

use crate::entities::user;
use crate::entities::user::User;
use crate::handlers::FensterError;

pub async fn get_user(
    State(AppInject { postgres_pool, .. }): State<AppInject>,
    Path(id): Path<String>,
) -> Result<Json<User>, FensterError> {
    let exists = user::exists(&id, &postgres_pool).await.unwrap_or(false);

    if !exists {
        return Err(FensterError::NotFound);
    }

    match user::fetch(&id, &postgres_pool).await {
        Ok(user) => Ok(Json(user)),
        Err(_error) => Err(FensterError::Internal)
    }
}
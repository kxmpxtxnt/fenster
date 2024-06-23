use axum::Json;
use anyhow::Result;
use axum_auth::AuthBearer;
use crate::handlers::FensterError;

pub async fn root(
    AuthBearer(token): AuthBearer
) -> Result<Json<String>, FensterError> {
    Ok(Json(token))
}

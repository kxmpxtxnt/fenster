use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub(crate) mod users;
pub(crate) mod root;
pub(crate) mod articles;
pub(crate) mod auth;

#[derive(Debug)]
pub enum FensterError {
    NotFound,
    Internal,
    Conflict,
    Unauthorized,
}

impl IntoResponse for FensterError {
    fn into_response(self) -> Response {
        let response = match self {
            FensterError::NotFound => StatusCode::NOT_FOUND,
            FensterError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            FensterError::Conflict => StatusCode::CONFLICT,
            FensterError::Unauthorized => StatusCode::UNAUTHORIZED
        };
        (response, response.canonical_reason().unwrap()).into_response()
    }
}
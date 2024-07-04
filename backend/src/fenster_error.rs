use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::fenster_error::FensterError::*;

#[derive(Debug)]
pub enum FensterError {
    NotFound(String),
    Internal(u16),
    Conflict(String),
    Unauthorized(String),
}

pub const OTHER_INTERNAL_ERROR: u16 = 111;
pub const POSTGRES_ERROR: u16 = 222;
pub const REDIS_ERROR: u16 = 444;

pub fn error(error: u16, i: u16) -> u16 {
    error * 10 + i
}

impl IntoResponse for FensterError {
    fn into_response(self) -> Response {
        match self {
            NotFound(message) => (StatusCode::NOT_FOUND, message),
            Conflict(message) => (StatusCode::CONFLICT, message),
            Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            Internal(code) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error. Code: {} | Please contact the responsible.", code)
            ),
        }.into_response()
    }
}
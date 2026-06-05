use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response}
};
use thiserror::Error;
use serde::Serialize;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing Authorization Headers")]
    MissingAuthorization,
    #[error("Invalid Credentials")]
    InvalidCredentials,
    #[error("Asset Does Not Exists")]
    AssetDoesNotExists,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
       let error_response = ErrorResponse {
           error: self.to_string()
       };
        
        let status = match self {  
            Self::MissingAuthorization => StatusCode::BAD_REQUEST,
            Self::AssetDoesNotExists => StatusCode::NOT_FOUND,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR
        };

        (status, Json(error_response)).into_response()
    }
}
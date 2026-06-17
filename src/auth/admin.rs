use crate::app::AppState;
use crate::error::AppError;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

const ADMIN_SECRET_KEY: &str = "admin";

#[derive(Debug)]
pub struct Admin;

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(auth) = parts.headers.get(AUTHORIZATION) else {
            return Err(AppError::MissingAuthorization);
        };

        if auth == ADMIN_SECRET_KEY {
            Ok(Admin)
        } else {
            Err(AppError::InvalidCredentials)
        }
    }
}

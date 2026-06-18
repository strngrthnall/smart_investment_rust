use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use smart_investment_rust::error::AppError;

async fn body_string(response: Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("failed to read body");
    String::from_utf8(bytes.to_vec()).expect("body is not utf-8")
}

#[tokio::test]
async fn test_missing_authorization_maps_to_400() {
    let response = AppError::MissingAuthorization.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    insta::assert_snapshot!(body_string(response).await);
}

#[tokio::test]
async fn test_invalid_credentials_maps_to_401() {
    let response = AppError::InvalidCredentials.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    insta::assert_snapshot!(body_string(response).await);
}

#[tokio::test]
async fn test_asset_does_not_exists_maps_to_404() {
    let response = AppError::AssetDoesNotExists.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    insta::assert_snapshot!(body_string(response).await);
}

#[tokio::test]
async fn test_user_does_not_exists_maps_to_404() {
    let response = AppError::UserDoesNotExists.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    insta::assert_snapshot!(body_string(response).await);
}

#[tokio::test]
async fn test_username_taken_maps_to_409() {
    let response = AppError::UserNameTaken.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    insta::assert_snapshot!(body_string(response).await);
}

#[tokio::test]
async fn test_database_error_maps_to_500() {
    let response = AppError::Database(sqlx::Error::RowNotFound).into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    insta::assert_snapshot!(body_string(response).await);
}

#[tokio::test]
async fn test_jwt_error_maps_to_500() {
    // Obtain a real `AppError::Jwt` by parsing a malformed token.
    let err =
        smart_investment_rust::auth::user::User::from_auth_token("not-a-valid-token").unwrap_err();
    assert!(matches!(err, AppError::Jwt(_)));
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    insta::assert_snapshot!(body_string(response).await);
}

#[tokio::test]
async fn test_reqwest_error_maps_to_500() {
    // A malformed URL produces a reqwest error without any network I/O.
    let reqwest_err = reqwest::get("not-a-valid-url").await.unwrap_err();
    let response = AppError::Reqwest(reqwest_err).into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

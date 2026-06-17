use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, Request},
};
use smart_investment_rust::{
    app::AppState,
    auth::admin::Admin,
    error::AppError,
};
use sqlx::PgPool;

#[sqlx::test]
async fn test_admin_extraction_missing_header(db: PgPool) {
    let state = AppState { db };
    let req = Request::builder().body(()).unwrap();
    let (mut parts, _) = req.into_parts();
    let err = Admin::from_request_parts(&mut parts, &state)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::MissingAuthorization));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_admin_extraction_invalid_credentials(db: PgPool) {
    let state = AppState { db };
    let req = Request::builder()
        .header(AUTHORIZATION, "invalid_secret")
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let err = Admin::from_request_parts(&mut parts, &state)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::InvalidCredentials));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_admin_extraction_success(db: PgPool) {
    let state = AppState { db };
    let req = Request::builder()
        .header(AUTHORIZATION, "admin")
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let admin = Admin::from_request_parts(&mut parts, &state)
        .await
        .expect("should extract Admin");
    insta::assert_debug_snapshot!(admin);
}

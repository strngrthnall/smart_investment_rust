use axum::extract::FromRequestParts;
use axum::http::{header::COOKIE, Request};
use smart_investment_rust::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    repository::Repository,
};
use sqlx::PgPool;

#[sqlx::test]
async fn test_check_new_user_success(db: PgPool) {
    let repository = Repository::from(db);
    let unauth = UnauthenticatedUser::new("new_user".to_string(), "pass".to_string());
    unauth
        .check_new_user(&repository)
        .await
        .expect("username should be free");
}

#[sqlx::test]
async fn test_check_new_user_already_exists(db: PgPool) {
    let repository = Repository::from(db);
    let unauth = UnauthenticatedUser::new("existing_user".to_string(), "pass".to_string());
    unauth
        .register(&repository)
        .await
        .expect("registration should succeed");

    let duplicate_check =
        UnauthenticatedUser::new("existing_user".to_string(), "other_pass".to_string());
    let err = duplicate_check
        .check_new_user(&repository)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::UserNameTaken));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_user_registration_success(db: PgPool) {
    let repository = Repository::from(db);
    let unauth = UnauthenticatedUser::new("reg_user".to_string(), "pass".to_string());
    let user = unauth
        .register(&repository)
        .await
        .expect("registration should succeed");
    assert_eq!(user.username(), "reg_user");
    insta::assert_json_snapshot!(user);
}

#[sqlx::test]
async fn test_user_registration_duplicate(db: PgPool) {
    let repository = Repository::from(db);
    let unauth = UnauthenticatedUser::new("reg_dup".to_string(), "pass".to_string());
    unauth
        .register(&repository)
        .await
        .expect("first registration should succeed");

    let duplicate = UnauthenticatedUser::new("reg_dup".to_string(), "other_pass".to_string());
    let err = duplicate.register(&repository).await.unwrap_err();
    assert!(matches!(err, AppError::UserNameTaken));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_user_authentication_success(db: PgPool) {
    let repository = Repository::from(db);
    let unauth = UnauthenticatedUser::new("auth_user".to_string(), "pass123".to_string());
    unauth
        .register(&repository)
        .await
        .expect("registration should succeed");

    let unauth_login = UnauthenticatedUser::new("auth_user".to_string(), "pass123".to_string());
    let user = unauth_login
        .authenticate(&repository)
        .await
        .expect("auth should succeed");
    assert_eq!(user.username(), "auth_user");
    insta::assert_json_snapshot!(user);
}

#[sqlx::test]
async fn test_user_authentication_invalid_password(db: PgPool) {
    let repository = Repository::from(db);
    let unauth = UnauthenticatedUser::new("auth_wrong_pw".to_string(), "pass123".to_string());
    unauth
        .register(&repository)
        .await
        .expect("registration should succeed");

    let unauth_login =
        UnauthenticatedUser::new("auth_wrong_pw".to_string(), "wrong_pass".to_string());
    let err = unauth_login.authenticate(&repository).await.unwrap_err();
    assert!(matches!(err, AppError::InvalidCredentials));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_user_authentication_non_existent(db: PgPool) {
    let repository = Repository::from(db);
    let unauth = UnauthenticatedUser::new("non_existent".to_string(), "pass123".to_string());
    let err = unauth.authenticate(&repository).await.unwrap_err();
    assert!(matches!(err, AppError::UserDoesNotExists));
    insta::assert_debug_snapshot!(err);
}

#[test]
fn test_id_getter() {
    let user = User::new(42, "marcos".to_string());
    assert_eq!(user.id(), 42);
    assert_eq!(user.username(), "marcos");
    insta::assert_json_snapshot!(user);
}

#[test]
fn test_auth_token_round_trip() {
    let token = User::new(7, "alice".to_string())
        .auth_token()
        .expect("should issue a token");

    let user = User::from_auth_token(&token).expect("should decode the token");
    assert_eq!(user.id(), 7);
    assert_eq!(user.username(), "alice");
    insta::assert_json_snapshot!(user);
}

#[test]
fn test_from_auth_token_invalid() {
    let err = User::from_auth_token("not-a-valid-token").unwrap_err();
    assert!(matches!(err, AppError::Jwt(_)));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_user_extractor_missing_token(db: PgPool) {
    let state = AppState { db };
    let req = Request::builder().body(()).unwrap();
    let (mut parts, _) = req.into_parts();
    let err = User::from_request_parts(&mut parts, &state)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::MissingAuthorization));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_user_extractor_invalid_token(db: PgPool) {
    let state = AppState { db };
    let req = Request::builder()
        .header(COOKIE, "token=garbage")
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let err = User::from_request_parts(&mut parts, &state)
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Jwt(_)));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_user_extractor_valid_token(db: PgPool) {
    let state = AppState { db };
    let token = User::new(7, "alice".to_string())
        .auth_token()
        .expect("should issue a token");

    let req = Request::builder()
        .header(COOKIE, format!("token={token}"))
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let user = User::from_request_parts(&mut parts, &state)
        .await
        .expect("should extract user from cookie");
    assert_eq!(user.id(), 7);
    assert_eq!(user.username(), "alice");
    insta::assert_json_snapshot!(user);
}

#[sqlx::test]
async fn test_option_user_extractor_none(db: PgPool) {
    let state = AppState { db };
    let req = Request::builder().body(()).unwrap();
    let (mut parts, _) = req.into_parts();
    let maybe_user = <Option<User> as FromRequestParts<AppState>>::from_request_parts(
        &mut parts, &state,
    )
        .await
        .expect("Option extractor is infallible");
    assert!(maybe_user.is_none());
    insta::assert_json_snapshot!(maybe_user);
}

#[sqlx::test]
async fn test_option_user_extractor_some(db: PgPool) {
    let state = AppState { db };
    let token = User::new(7, "alice".to_string())
        .auth_token()
        .expect("should issue a token");

    let req = Request::builder()
        .header(COOKIE, format!("token={token}"))
        .body(())
        .unwrap();
    let (mut parts, _) = req.into_parts();
    let maybe_user = <Option<User> as FromRequestParts<AppState>>::from_request_parts(
        &mut parts, &state,
    )
        .await
        .expect("Option extractor is infallible");
    insta::assert_json_snapshot!(maybe_user);
    let user = maybe_user.expect("should contain a user");
    assert_eq!(user.username(), "alice");
}

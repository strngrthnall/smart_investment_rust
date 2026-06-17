use axum::Form;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use smart_investment_rust::{
    auth::user::User,
    error::AppError,
    repository::Repository,
    routes::frontend::{
        create_user, create_user_page, index, login, login_page, CreateUserForm, LoginForm,
    },
};
use sqlx::PgPool;

#[tokio::test]
async fn test_login_page_renders() {
    let res = login_page().await.expect("should render login page");
    assert!(res.0.contains("Smart Investment - Login"));
    assert!(res.0.contains("Usuário"));
    insta::assert_snapshot!(res.0);
}

#[tokio::test]
async fn test_create_user_page_renders() {
    let res = create_user_page()
        .await
        .expect("should render create user page");
    assert!(res.0.contains("Smart Investment"));
    assert!(res.0.contains("Senha"));
    insta::assert_snapshot!(res.0);
}

#[sqlx::test]
async fn test_create_user_success(db: PgPool) {
    let request = CreateUserForm {
        username: "test_frontend_user".to_string(),
        password: "test_password_123".to_string(),
    };
    let response = create_user(Repository::from(db), Form(request))
        .await
        .expect("should register user successfully");
    assert!(response.0.contains("Usuário criado com sucesso"));
    insta::assert_snapshot!(response.0);
}

#[sqlx::test]
async fn test_create_user_username_taken(db: PgPool) {
    // Pre-register user
    let first_request = CreateUserForm {
        username: "test_frontend_user".to_string(),
        password: "test_password_123".to_string(),
    };
    let _ = create_user(Repository::from(db.clone()), Form(first_request))
        .await
        .expect("first registration should succeed");

    // Try duplicate
    let duplicate_request = CreateUserForm {
        username: "test_frontend_user".to_string(),
        password: "some_other_password".to_string(),
    };
    let error_response = create_user(Repository::from(db), Form(duplicate_request))
        .await
        .expect("should render page even when username is taken");
    assert!(error_response
        .0
        .contains("This username is already registered"));
    insta::assert_snapshot!(error_response.0);
}

#[sqlx::test]
async fn test_login_success(db: PgPool) {
    // Pre-create the user
    let register_request = CreateUserForm {
        username: "login_flow_user".to_string(),
        password: "correct_password".to_string(),
    };
    let _ = create_user(Repository::from(db.clone()), Form(register_request))
        .await
        .expect("should register user");

    // Attempt login
    let login_success_request = LoginForm {
        username: "login_flow_user".to_string(),
        password: "correct_password".to_string(),
    };
    let response = login(
        Repository::from(db),
        CookieJar::new(),
        Form(login_success_request),
    )
        .await
        .expect("login should succeed")
        .into_response();
    assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(axum::http::header::LOCATION)
            .unwrap(),
        "/"
    );
    let cookie_header = response
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .unwrap();
    assert!(cookie_header.to_str().unwrap().contains("token="));
    insta::assert_debug_snapshot!(response.status());
}

#[sqlx::test]
async fn test_login_user_not_found(db: PgPool) {
    let login_non_existent = LoginForm {
        username: "non_existent_user".to_string(),
        password: "any_password".to_string(),
    };
    let response_non_existent = login(
        Repository::from(db),
        CookieJar::new(),
        Form(login_non_existent),
    )
        .await
        .expect("should render error page for non-existent user")
        .into_response();
    assert_eq!(response_non_existent.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(response_non_existent.into_body(), 1024 * 1024)
        .await
        .expect("failed to read body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("body is not utf-8");
    assert!(body_str.contains("Usuário não existe"));
    insta::assert_snapshot!(body_str);
}

#[sqlx::test]
async fn test_login_wrong_password(db: PgPool) {
    // Pre-create the user
    let register_request = CreateUserForm {
        username: "login_flow_user".to_string(),
        password: "correct_password".to_string(),
    };
    let _ = create_user(Repository::from(db.clone()), Form(register_request))
        .await
        .expect("should register user");

    // Attempt login with wrong password
    let login_wrong_pw = LoginForm {
        username: "login_flow_user".to_string(),
        password: "wrong_password".to_string(),
    };
    let err = login(Repository::from(db), CookieJar::new(), Form(login_wrong_pw))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::InvalidCredentials));
    insta::assert_debug_snapshot!(err);
}

#[sqlx::test]
async fn test_index_no_user_redirects(_db: PgPool) {
    let response = index(None).await.expect("index should succeed");
    assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(axum::http::header::LOCATION)
            .unwrap(),
        "/login"
    );
    insta::assert_debug_snapshot!(response.status());
}

#[sqlx::test]
async fn test_index_with_user_shows_greeting(_db: PgPool) {
    let user = User::new(42, "marcos".to_string());
    let response = index(Some(user)).await.expect("index should succeed");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("failed to read body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("body is not utf-8");
    assert!(body_str.contains("hello, marcos"));
    insta::assert_snapshot!(body_str);
}

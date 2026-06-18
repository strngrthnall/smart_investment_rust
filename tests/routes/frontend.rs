use axum::Form;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use smart_investment_rust::{
    auth::user::User,
    error::AppError,
    repository::Repository,
    routes::frontend::{
        assets, create_user, create_user_page, index, login, login_page, logout, purchase_asset,
        CreateUserForm, LoginForm, PurchaseAssetForm,
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
    let response = index(None)
        .await
        .expect("index should succeed")
        .into_response();
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
async fn test_index_with_user_redirects_to_assets(_db: PgPool) {
    let user = User::new(42, "marcos".to_string());
    let response = index(Some(user))
        .await
        .expect("index should succeed")
        .into_response();
    assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(axum::http::header::LOCATION)
            .unwrap(),
        "/assets"
    );
    insta::assert_debug_snapshot!(response.status());
}

#[tokio::test]
async fn test_logout_redirects_and_clears_cookie() {
    // Build the jar from request headers so "token" is an original cookie; only
    // then does removing it emit a clearing Set-Cookie.
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::COOKIE,
        "token=existing-token".parse().unwrap(),
    );
    let jar = CookieJar::from_headers(&headers);
    let response = logout(jar).await.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(axum::http::header::LOCATION)
            .unwrap(),
        "/login"
    );
    // Removing the cookie emits a clearing Set-Cookie for "token".
    let set_cookie = response
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .unwrap();
    assert!(set_cookie.to_str().unwrap().contains("token="));
    insta::assert_debug_snapshot!(response.status());
}

#[sqlx::test]
async fn test_assets_page_empty(db: PgPool) {
    let repository = Repository::from(db);
    let user_record = repository
        .add_user("portfolio_user", "hash")
        .await
        .expect("should insert user");
    // Assets are available to buy, but the user owns none yet.
    repository
        .create_asset("Bitcoin".to_string(), 100.0)
        .await
        .expect("should insert asset");
    repository
        .create_asset("Ethereum".to_string(), 50.0)
        .await
        .expect("should insert asset");

    let user = User::new(user_record.id, "portfolio_user".to_string());
    let response = assets(repository, user)
        .await
        .expect("should render assets page");

    assert!(response.0.contains("portfolio_user"));
    assert!(response.0.contains("Você ainda não possui nenhum ativo"));
    assert!(response.0.contains("Bitcoin"));
    assert!(response.0.contains("Ethereum"));
    insta::assert_snapshot!(response.0);
}

#[sqlx::test]
async fn test_assets_page_with_holdings(db: PgPool) {
    let repository = Repository::from(db.clone());
    let user_record = repository
        .add_user("holder", "hash")
        .await
        .expect("should insert user");
    let asset = repository
        .create_asset("Bitcoin".to_string(), 100.0)
        .await
        .expect("should insert asset");

    // Insert the holding with a fixed timestamp so the rendered purchase
    // history is deterministic for snapshotting.
    sqlx::query(
        "INSERT INTO owned_assets (user_id, asset_id, bought_for, quantity_owned, timestamp)
            VALUES ($1, $2, $3, $4, $5::timestamptz);",
    )
    .bind(user_record.id)
    .bind(asset.id)
    .bind(80.0_f64)
    .bind(2.0_f64)
    .bind("2024-01-15T12:00:00Z")
    .execute(&db)
    .await
    .expect("should insert owned asset");

    let user = User::new(user_record.id, "holder".to_string());
    let response = assets(repository, user)
        .await
        .expect("should render assets page");

    assert!(response.0.contains("Bitcoin"));
    // total_value = current 100 * 2 units = 200
    assert!(response.0.contains("R$ 200.00"));
    // value_delta = (100 - 80) * 2 = +40
    assert!(response.0.contains("R$ +40.00"));
    insta::assert_snapshot!(response.0);
}

#[sqlx::test]
async fn test_purchase_asset_success(db: PgPool) {
    let repository = Repository::from(db.clone());
    let user_record = repository
        .add_user("buyer", "hash")
        .await
        .expect("should insert user");
    let asset = repository
        .create_asset("Bitcoin".to_string(), 100.0)
        .await
        .expect("should insert asset");

    let form = PurchaseAssetForm {
        asset_id: asset.id,
        unit_value: 90.0,
        quantity: 3.0,
    };
    let user = User::new(user_record.id, "buyer".to_string());
    let response = purchase_asset(repository, user, Form(form))
        .await
        .expect("purchase should succeed")
        .into_response();

    assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get(axum::http::header::LOCATION)
            .unwrap(),
        "/assets"
    );

    // The holding must have been persisted.
    let owned = Repository::from(db)
        .list_owned_assets(user_record.id)
        .await
        .expect("query should succeed");
    assert_eq!(owned.len(), 1);
    assert_eq!(owned[0].quantity_owned, 3.0);

    insta::assert_debug_snapshot!(response.status());
}

#[sqlx::test]
async fn test_purchase_asset_invalid_asset(db: PgPool) {
    let repository = Repository::from(db);
    let user_record = repository
        .add_user("buyer2", "hash")
        .await
        .expect("should insert user");

    let form = PurchaseAssetForm {
        asset_id: 999_999,
        unit_value: 90.0,
        quantity: 1.0,
    };
    let user = User::new(user_record.id, "buyer2".to_string());
    let err = match purchase_asset(repository, user, Form(form)).await {
        Ok(_) => panic!("purchasing a non-existent asset should fail"),
        Err(err) => err,
    };
    assert!(matches!(err, AppError::Database(_)));

    // Snapshot the violated constraint name (stable across Postgres builds).
    let constraint = match &err {
        AppError::Database(db_err) => db_err
            .as_database_error()
            .and_then(|inner| inner.constraint())
            .map(str::to_string),
        _ => None,
    };
    insta::assert_debug_snapshot!(constraint);
}

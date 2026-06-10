use askama::Template;
use axum::{response::Html, Router, routing::get, Form};
use serde::Deserialize;
use crate::{
    app::AppState,
    error::AppError,
    repository::Repository
};

use crate::auth::user::UnauthenticatedUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/create_user", get(create_user_page).post(create_user))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage {
    error_message: Option<String>,
}

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage { error_message: None }.render()?;
    Ok(Html(html))
}

#[derive(Template)]
#[template(path = "create_user.html")]
struct CreateUserPage {
    error_message: Option<String>,
}

async fn create_user_page() -> Result<Html<String>, AppError> {
    let html = CreateUserPage { error_message: None }.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct CreateUserForm {
    username: String,
    password: String,
}

async fn login(repository: Repository, Form(request): Form<LoginForm>) -> Result<Html<String>, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExists) => {
            let html = LoginPage {
                error_message: Some("Usuário não existe".to_string()),
            }
            .render()?;
            return Ok(Html(html));
        }
        Err(other_err) => return Err(other_err),
    };
    Ok(Html(user.username().clone()))
}

async fn create_user(
    repository: Repository,
    Form(request): Form<CreateUserForm>,
) -> Result<Html<String>, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);

    if let Err(err) = unauth_user.check_new_user(&repository).await {
        let html = CreateUserPage {
            error_message: Some(err.to_string()),
        }
        .render()?;
        return Ok(Html(html));
    }

    let user = unauth_user.register(&repository).await?;
    Ok(Html(user.username().clone()))
}
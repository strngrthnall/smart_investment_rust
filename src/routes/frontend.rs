use crate::{app::AppState, error::AppError, repository::Repository};
use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use serde::Deserialize;

use crate::auth::user::{UnauthenticatedUser, User};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/create_user", get(create_user_page).post(create_user))
        .route("/", get(index))
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginPage {
    pub error_message: Option<String>,
    pub success_message: Option<String>,
}

pub async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage {
        error_message: None,
        success_message: None,
    }
    .render()?;
    Ok(Html(html))
}

#[derive(Template)]
#[template(path = "create_user.html")]
pub struct CreateUserPage {
    pub error_message: Option<String>,
}

pub async fn create_user_page() -> Result<Html<String>, AppError> {
    let html = CreateUserPage {
        error_message: None,
    }
    .render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateUserForm {
    pub username: String,
    pub password: String,
}

pub async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<Response, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExists) => {
            let html = LoginPage {
                error_message: Some("Usuário não existe".to_string()),
                success_message: None,
            }
            .render()?;
            return Ok(Html(html).into_response());
        }
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token()?;

    let cookie = Cookie::build(("token", token)).http_only(true);

    Ok((jar.add(cookie), Redirect::to("/")).into_response())
}

pub async fn create_user(
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

    let _user = unauth_user.register(&repository).await?;
    let login_page_html = LoginPage {
        error_message: None,
        success_message: Some("Usuário criado com sucesso".to_string()),
    }
    .render()?;
    Ok(Html(login_page_html))
}

pub async fn index(maybe_user: Option<User>) -> Result<Response, AppError> {
    match maybe_user {
        Some(user) => Ok(Html(format!("hello, {}", user.username())).into_response()),
        None => Ok(Redirect::to("/login").into_response()),
    }
}

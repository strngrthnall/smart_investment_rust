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
use tokio::try_join;
use crate::auth::user::{UnauthenticatedUser, User};
use crate::models::{Asset, OwnedAsset};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/create_user", get(create_user_page).post(create_user))
        .route("/logout", get(logout))
        .route("/assets", get(assets).post(purchase_asset))
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

pub async fn index(maybe_user: Option<User>) -> Result<Redirect, AppError> {
    match maybe_user {
        Some(_) => Ok(Redirect::to("/assets")),
        None => Ok(Redirect::to("/login")),
    }
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove("token"), Redirect::to("/login"))
}

#[derive(Template)]
#[template(path = "assets.html")]
pub struct AssetsPage {
    owned_assets: Vec<OwnedAsset>,
    available_assets: Vec<Asset>,
    user: User,
    total_value: f64,
    total_delta: f64,
}

pub async fn assets(repository: Repository, user: User) -> Result<Html<String>, AppError> {
    // Sync assets cotações (will check throttling inside)
    crate::routes::api::sync_assets_with_api(&repository).await?;

    let (owned_assets, available_assets) = try_join!(
        repository.list_owned_assets(user.id()),
        repository.list_assets()
    )?;

    let mut total_value = 0.0;
    let mut total_delta = 0.0;
    for asset in &owned_assets {
        total_value += asset.unit_value * asset.quantity_owned;
        total_delta += asset.value_delta;
    }

    let html = AssetsPage {
        owned_assets,
        available_assets,
        user,
        total_value,
        total_delta,
    }.render()?;

    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct PurchaseAssetForm {
    pub asset_id: i64,
    pub unit_value: f64,
    pub quantity: f64,
}

pub async fn purchase_asset(
    repository: Repository,
    user: User,
    Form(request): Form<PurchaseAssetForm>
) -> Result<Redirect, AppError> {
    repository
        .insert_owned_asset(
            user.id(),
            request.asset_id,
            request.quantity,
            request.unit_value,
        ).await?;

    Ok(Redirect::to("/assets"))
}
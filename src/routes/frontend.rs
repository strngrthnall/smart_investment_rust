use crate::{app::AppState, error::AppError, repository::Repository};
use askama::Template;
use axum::{Form, Router, response::Html, routing::get};
use serde::Deserialize;

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
    let html = LoginPage {
        error_message: None,
    }
    .render()?;
    Ok(Html(html))
}

#[derive(Template)]
#[template(path = "create_user.html")]
struct CreateUserPage {
    error_message: Option<String>,
}

async fn create_user_page() -> Result<Html<String>, AppError> {
    let html = CreateUserPage {
        error_message: None,
    }
    .render()?;
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

async fn login(
    repository: Repository,
    Form(request): Form<LoginForm>,
) -> Result<Html<String>, AppError> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::AppError;
    use crate::repository::Repository;
    use axum::Form;
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
        assert_eq!(response.0, "test_frontend_user");
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
        assert!(
            error_response
                .0
                .contains("This username is already registered")
        );
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
        let response = login(Repository::from(db), Form(login_success_request))
            .await
            .expect("login should succeed");
        assert_eq!(response.0, "login_flow_user");
        insta::assert_snapshot!(response.0);
    }

    #[sqlx::test]
    async fn test_login_user_not_found(db: PgPool) {
        let login_non_existent = LoginForm {
            username: "non_existent_user".to_string(),
            password: "any_password".to_string(),
        };
        let response_non_existent = login(Repository::from(db), Form(login_non_existent))
            .await
            .expect("should render error page for non-existent user");
        assert!(response_non_existent.0.contains("Usuário não existe"));
        insta::assert_snapshot!(response_non_existent.0);
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
        let err = login(Repository::from(db), Form(login_wrong_pw))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::InvalidCredentials));
        insta::assert_debug_snapshot!(err);
    }
}

use std::convert::Infallible;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use jwt_simple::algorithms::MACLike;
use jwt_simple::claims::Claims;
use jwt_simple::prelude::{Duration, HS256Key};
use password_auth::VerifyError;

use crate::app::AppState;
use crate::{error::AppError, repository::Repository};

const SECRET_KEY: &[u8] = b"0100010001101111011000110110010100100000011011011100001110100011\
                            0110010100101100001000000110010001101111011000110110010100100000\
                            0110110111000011101000110110010100101100001000000110110101100001\
                            0110111001100100011001010010000001110011011101010110000100100000\
                            0110001101110010011010010110000101101110110000111010011101100001\
                            0010000001110000011000010111001001100001001000000110110101101001\
                            0110110100101100001000000111000001101111011010010111001100100000\
                            0110111101110011001000000111000001100101011000110110000101100100\
                            0110111101110011001000000110010001101111011100110010000001101001\
                            0110111001100100011010010110011101101110011011110111001100100000\
                            0110010001100101011101100110010101101101001000000111001101100101\
                            0111001000100000011000100110000101110100011010010111101001100001\
                            0110010001101111011100110010000001100101011011010010000001110011\
                            0110000101101110011001110111010101100101001000000110010100100000\
                            0110110101100101011001000110111100101110";

pub struct UnauthenticatedUser {
    username: String,
    password: String,
}

impl UnauthenticatedUser {
    pub fn new(username: String, password: String) -> UnauthenticatedUser {
        UnauthenticatedUser { username, password }
    }

    pub async fn authenticate(&self, repository: &Repository) -> Result<User, AppError> {
        let user_record = match repository.get_user_by_name(&self.username).await? {
            Some(user_record) => user_record,
            None => return Err(AppError::UserDoesNotExists),
        };

        match password_auth::verify_password(&self.password, &user_record.password_hash) {
            Ok(_) => Ok(User::new(user_record.id, user_record.username)),
            Err(VerifyError::PasswordInvalid) => Err(AppError::InvalidCredentials),
            Err(VerifyError::Parse(err)) => panic!("Hashing algorithm failed: {err}"),
        }
    }

    pub async fn check_new_user(&self, repository: &Repository) -> Result<(), AppError> {
        match repository.get_user_by_name(&self.username).await? {
            Some(_) => Err(AppError::UserNameTaken),
            None => Ok(()),
        }
    }

    pub async fn register(self, repository: &Repository) -> Result<User, AppError> {
        let password_hash = password_auth::generate_hash(self.password);

        let user_record = match repository.add_user(&self.username, &password_hash).await {
            Ok(user_record) => user_record,
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                return Err(AppError::UserNameTaken);
            }
            Err(err) => return Err(AppError::Database(err)),
        };

        Ok(User::new(user_record.id, user_record.username))
    }
}

#[allow(unused)]
#[derive(Debug, serde::Serialize)]
pub struct User {
    id: i64,
    username: String,
}

impl User {
    pub fn new(id: i64, username: String) -> Self {
        User { id, username }
    }

    pub const fn username(&self) -> &String {
        &self.username
    }

    pub const fn id(&self) -> &i64 {
        &self.id
    }

    pub fn auth_token(self) -> Result<String, AppError> {
        let key = HS256Key::from_bytes(SECRET_KEY);
        let claims = Claims::with_custom_claims(UserClaims::from(self), Duration::from_mins(10));
        let token = key.authenticate(claims)?;
        Ok(token)
    }

    pub fn from_auth_token(token: &str) -> Result<Self, AppError> {
        let key = HS256Key::from_bytes(SECRET_KEY);
        let claims: UserClaims = key.verify_token(token, None)?.custom;
        Ok(Self::new(claims.id, claims.username))
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let token = match jar.get("token") {
            Some(token) => token.value(),
            None => return Err(AppError::MissingAuthorization),
        };

        User::from_auth_token(token)
    }
}

impl FromRequestParts<AppState> for Option<User> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts, 
        state: &AppState
    ) -> Result<Self, Self::Rejection> {
        Ok(User::from_request_parts(parts, state).await.ok())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserClaims {
    id: i64,
    username: String,
}

impl From<User> for UserClaims {
    fn from(User { id, username }: User) -> Self {
        Self { id, username }
    }
}

use password_auth::VerifyError;
use crate::{
    error::AppError,
    repository::Repository
};

pub struct UnauthenticatedUser {
    username: String,
    password: String
}



impl UnauthenticatedUser {
    pub(crate) fn new(username: String, password: String) -> UnauthenticatedUser {
        UnauthenticatedUser {username, password}
    }

    pub async fn authenticate(&self, repository: &Repository) -> Result<User, AppError> {
        let user_record = match repository.get_user_by_name(&self.username).await? {
            Some(user_record) => user_record,
            None => return Err(AppError::UserDoesNotExists),
        };

        match password_auth::verify_password(&self.password, &user_record.password_hash) {
            Ok(_) => Ok(User::new(user_record.id, user_record.username)),
            Err(VerifyError::PasswordInvalid) => Err(AppError::InvalidCredentials),
            Err(VerifyError::Parse(err)) => panic!("Hashing algorithm failed: {err}")
        }
    }
    
    pub async fn check_new_user(&self, repository: &Repository) -> Result<(), AppError> {
        match repository.get_user_by_name(&self.username).await? {
            Some(_) => Err(AppError::UserNameTaken),
            None => Ok(()),
        }
    }

    pub async fn register(self, repository: &Repository) -> Result<User, AppError>{
        let password_hash = password_auth::generate_hash(self.password);

        let user_record = match repository.add_user(&self.username, &password_hash).await {
            Ok(user_record) => user_record,
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                return Err(AppError::UserNameTaken);
            }
            Err(err) => return Err(AppError::Database(err))
        };

        Ok(User::new(user_record.id, user_record.username))
    }
}

#[allow(unused)]
pub struct User {
    id: i64,
    username: String,

}

impl User {
    fn new(id: i64, username: String) -> Self {
        User {id, username}
    }

    pub const fn username(&self) -> &String {
        &self.username
    }

    pub const fn id(&self) -> &i64 {
        &self.id
    }

}
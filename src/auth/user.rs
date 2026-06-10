use crate::{error::AppError, repository::Repository};
use password_auth::VerifyError;

pub struct UnauthenticatedUser {
    username: String,
    password: String,
}

impl UnauthenticatedUser {
    pub(crate) fn new(username: String, password: String) -> UnauthenticatedUser {
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
    fn new(id: i64, username: String) -> Self {
        User { id, username }
    }

    pub const fn username(&self) -> &String {
        &self.username
    }

    pub const fn id(&self) -> &i64 {
        &self.id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::AppError;
    use crate::repository::Repository;
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
        let err = unauth_login
            .authenticate(&repository)
            .await
            .unwrap_err();
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
}

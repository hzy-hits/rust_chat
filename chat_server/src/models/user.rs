use std::mem;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use chat_core::{ChatUser, User};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

impl AppState {
    #[allow(dead_code)]
    pub async fn find_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>, AppError> {
        let ret = sqlx::query_as(
            "SELECT id,ws_id, username, email,created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(ret)
    }

    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        let password_hash = hash_password(&input.password)?;

        // start a transaction
        let mut tx = self.pg_pool.begin().await?;

        // search for the workspace
        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        // try to insert the user
        let user_result = sqlx::query_as::<_, User>(
            r#"
        INSERT INTO users (ws_id, email, username, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, ws_id, username, email, created_at
        "#,
        )
        .bind(ws.id)
        .bind(&input.email)
        .bind(&input.username)
        .bind(password_hash)
        .fetch_one(&mut *tx)
        .await;

        // handle the result
        let user = match user_result {
            Ok(user) => user,
            Err(err) => {
                tx.rollback().await?; // rollback the transaction
                match err {
                    sqlx::Error::Database(db_err) => {
                        // PostgreSQL unique constraint violation
                        if db_err.code().as_deref() == Some("23505")
                            && db_err.constraint() == Some("users_email_key")
                        {
                            return Err(AppError::EmailAlreadyExists(input.email.clone()));
                        }
                        return Err(AppError::SqlxError(sqlx::Error::Database(db_err)));
                    }
                    err => return Err(AppError::SqlxError(err)),
                }
            }
        };

        // if the workspace owner is not set, set it to the current user
        if ws.owner_id == 0 {
            sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
                .bind(user.id)
                .bind(ws.id)
                .execute(&mut *tx)
                .await?;
        }

        //submits the transaction
        tx.commit().await?;

        Ok(user)
    }

    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "
            SELECT id,ws_id,username,email,password_hash,created_at FROM users WHERE email = $1
            ",
        )
        .bind(&input.email)
        .fetch_optional(&self.pg_pool)
        .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl AppState {
    pub async fn fetch_chat_user_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, username, email
        FROM users
        WHERE id = ANY($1)
        "#,
        )
        .bind(ids)
        .fetch_all(&self.pg_pool)
        .await?;
        Ok(users)
    }
    #[allow(dead_code)]
    pub async fn fetch_chat_users(&self, ws_id: i64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, username, email
        FROM users
        WHERE ws_id = $1
        "#,
        )
        .bind(ws_id)
        .fetch_all(&self.pg_pool)
        .await?;
        Ok(users)
    }
    #[allow(dead_code)]
    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            r#"
        SELECT id, ws_id, username, email, created_at
        FROM users
        WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(user)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();
    Ok(is_valid)
}

// #[cfg(test)]
// impl User {
//     pub fn new(id: i64, username: &str, email: &str) -> Self {
//         use chrono::Utc;
//         Self {
//             id,
//             ws_id: 0,
//             username: username.to_string(),
//             email: email.to_string(),
//             password_hash: None,
//             created_at: Utc::now(),
//         }
//     }
// }

#[cfg(test)]
impl CreateUser {
    pub fn new(ws: &str, username: &str, email: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            workspace: ws.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "hunter42";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(password, &password_hash)?);
        Ok(())
    }
    #[tokio::test]
    async fn find_user_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state.find_user_by_id(1).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.id, 1);
        Ok(())
    }
    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateUser::new("acme", "test", "test2@acme.org", "hunter42");
        let ret = state.create_user(&input).await;
        match ret {
            Err(AppError::EmailAlreadyExists(email)) => {
                assert_eq!(email, input.email);
            }
            _ => panic!("Expecting EmailAlreadyExists error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateUser::new("none", "test", "test@test.test", "hunter42");
        let user = state.create_user(&input).await?;
        assert_eq!(user.email, input.email);
        assert_eq!(user.username, input.username);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&input.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, input.email);
        assert_eq!(user.username, input.username);

        let input = SigninUser::new(&input.email, &input.password);
        let user = state.verify_user(&input).await?;
        assert!(user.is_some());

        Ok(())
    }
}

use chat_core::{Chat, ChatType, User};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

use crate::{AppError, AppState};

#[derive(Debug, Clone, FromRow, ToSchema, Serialize, Deserialize, PartialEq)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Option<Vec<i64>>,
    pub public: Option<bool>,
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_chat(
        &self,
        input: CreateChat,
        user_id: u64,
        ws_id: u64,
    ) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }
        // if user id is not in members, reject
        if !input.members.contains(&(user_id as i64)) {
            return Err(AppError::CreateChatError(
                "You must be a member of the chat".to_string(),
            ));
        }
        if let Some(name) = &input.name {
            if name.len() < 3 {
                return Err(AppError::CreateChatError(
                    "Chat name must have at least 3 characters".to_string(),
                ));
            }
        }

        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }
        let users = self.fetch_chat_user_by_ids(&input.members).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }
        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };
        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, chat_type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, chat_type, members, created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(&self.pg_pool)
        .await?;
        Ok(chat)
    }
    pub async fn fetch_chats(&self, user_id: u64, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, chat_type, members, created_at
            FROM chats
            WHERE ws_id = $1 AND $2 = ANY(members)
            "#,
        )
        .bind(ws_id as i64)
        .bind(user_id as i64)
        .fetch_all(&self.pg_pool)
        .await?;
        Ok(chats)
    }
    pub async fn get_chat_by_id(&self, id: u64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, chat_type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(chat)
    }

    pub async fn fetch_and_verify_chat(&self, chat_id: u64, user: &User) -> Result<Chat, AppError> {
        let chat = self
            .get_chat_by_id(chat_id)
            .await?
            .ok_or(AppError::NotFound(format!("Chat id {}", chat_id)))?;

        // check if user is a member of the chat
        if chat.ws_id != user.ws_id {
            return Err(AppError::Unauthorized(
                "You do not have permission to update this chat.".into(),
            ));
        }

        Ok(chat)
    }
    pub async fn apply_updates(&self, chat: &mut Chat, input: UpdateChat) -> Result<(), AppError> {
        if let Some(name) = input.name {
            chat.name = Some(name);
        }
        if let Some(members) = input.members {
            let len = members.len();
            if len < 2 {
                return Err(AppError::UpdateChatError(
                    "Chat must have at least 2 members".to_string(),
                ));
            }
            if len > 8 && chat.name.is_none() {
                return Err(AppError::UpdateChatError(
                    "Group chat with more than 8 members must have a name".to_string(),
                ));
            }
            let users = self.fetch_chat_user_by_ids(&members).await?;
            if users.len() != len {
                return Err(AppError::UpdateChatError(
                    "Some members do not exist".to_string(),
                ));
            }
            chat.members = members;
        }
        if let Some(public) = input.public {
            chat.chat_type = if public {
                ChatType::PublicChannel
            } else {
                ChatType::PrivateChannel
            };
        } else {
            // if no public field is provided, update chat type based on the number of members
            chat.chat_type = match (&chat.name, chat.members.len()) {
                (None, 2) => ChatType::Single,
                (None, _) => ChatType::Group,
                (Some(_), _) => chat.chat_type.clone(), // keep the current chat type
            };
        }
        sqlx::query(
            r#"
        UPDATE chats
        SET name = $1, type = $2, members = $3
        WHERE id = $4
        "#,
        )
        .bind(&chat.name)
        .bind(&chat.chat_type)
        .bind(&chat.members)
        .bind(chat.id)
        .execute(&self.pg_pool)
        .await?;
        Ok(())
    }

    pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
        let is_member = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 = ANY(members)
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(is_member.is_some())
    }
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::AppState;

    #[tokio::test]
    async fn create_single_chat_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = state
            .create_chat(input, 1, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.chat_type, ChatType::Single);
        Ok(())
    }
    #[tokio::test]
    async fn create_public_named_chat_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = state
            .create_chat(input, 1, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.chat_type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state
            .get_chat_by_id(1)
            .await
            .expect("get chat by id failed")
            .unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.unwrap(), "general");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 5);
        Ok(())
    }
    #[tokio::test]
    async fn chat_fetch_all_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state
            .fetch_chats(1, 1)
            .await
            .expect("fetch all chats failed");
        assert_eq!(chats.len(), 4);
        Ok(())
    }
    #[tokio::test]
    async fn chat_is_member_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let is_member = state
            .is_chat_member(1, 1)
            .await
            .expect("is chat member failed");
        assert!(is_member);
        let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
        assert!(!is_member);
        let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
        assert!(!is_member);
        let is_member = state.is_chat_member(2, 4).await.expect("is member failed");
        assert!(!is_member);

        Ok(())
    }
}

use crate::{AppError, AppState};
use chat_core::{ChatUser, Workspace};
impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name,owner_id)
            VALUES ($1,$2)
            RETURNING id,name,owner_id,created_at
            "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pg_pool)
        .await?;
        Ok(ws)
    }

    pub async fn fetch_all_chat_users(&self, id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
        SELECT id, username, email
        FROM users
        WHERE ws_id = $1 order by id
        "#,
        )
        .bind(id as i64)
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(users)
    }

    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as::<_, Workspace>(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(ws)
    }
    #[allow(dead_code)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pg_pool)
        .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Ok;

    #[tokio::test]
    async fn workspace_should_find_by_name() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await.unwrap();
        assert_eq!(ws.name, "test");
        Ok(())
    }
    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let users = state.fetch_all_chat_users(1).await.unwrap();
        assert_eq!(users.len(), 5);

        Ok(())
    }
}

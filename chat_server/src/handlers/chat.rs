use crate::{
    models::chat::{CreateChat, UpdateChat},
    AppError, AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::{Chat, User};
use sqlx::{Postgres, Transaction};
use tracing::info;

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.fetch_chats(user.ws_id as _).await?;

    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn create_chat_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id as _).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}
#[allow(dead_code)]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id as _).await?;
    match chat {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound(format!("chat id {id}"))),
    }
}

pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let mut chat = state.fetch_and_verify_chat(id, &user).await?;
    state.apply_updates(&mut chat, input).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn delete_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let mut tx: Transaction<'_, Postgres> = state.pg_pool.begin().await?;

    let chat = sqlx::query_as::<_, Chat>(
        r#"
            SELECT id, ws_id, name, chat_type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
    )
    .bind(id as i64)
    .fetch_optional(tx.as_mut())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Chat id {}", id)))?;

    if chat.ws_id != user.ws_id {
        return Err(AppError::Unauthorized(
            "You do not have permission to delete this chat.".into(),
        ));
    }

    sqlx::query(
        r#"
        DELETE FROM chats
        WHERE id = $1
        "#,
    )
    .bind(chat.id)
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    info!("Chat {} deleted successfully by user {}", id, user.ws_id);
    Ok(StatusCode::NO_CONTENT)
}

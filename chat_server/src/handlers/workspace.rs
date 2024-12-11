use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{AppError, AppState, User};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    println!("Handling /api/users request");
    let users = state.fetch_all_chat_users(user.ws_id as _).await?;
    Ok(Json(users))
}

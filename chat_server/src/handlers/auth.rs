use crate::{models::user::CreateUser, AppError, AppState, User};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input, &state.pg_pool).await?;
    let token = state.ek.sign(user)?;
    Ok((StatusCode::CREATED, token))
}

pub(crate) async fn signup_handler() -> impl IntoResponse {
    "signup"
}

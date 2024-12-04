use axum::response::IntoResponse;

pub(crate) async fn _send_message_handler() -> impl IntoResponse {
    "send_message"
}

pub(crate) async fn list_messages_handler() -> impl IntoResponse {
    "list_messages"
}

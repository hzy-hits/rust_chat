use axum::{
    extract::State,
    response::{sse::Event, Sse},
    Extension,
};
use chat_core::{Chat, Message, User};

use futures::Stream;
use jwt_simple::reexports::serde_json;
use pin_project::pin_project;
// use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::broadcast::{self};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::info;

use crate::{AppState, UserMap};
const CHANNEL_CAPACITY: usize = 256;

struct CleanupGuard {
    user_id: u64,
    users: UserMap,
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Some((_, _)) = self.users.remove(&self.user_id) {
            info!("Cleaned up user {} from notification system", self.user_id);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
    ChatNameUpdated(Chat),
}
#[pin_project]
struct WithCleanup<S> {
    #[pin]
    stream: S,
    _guard: CleanupGuard,
}

impl<S: Stream> Stream for WithCleanup<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.poll_next(cx)
    }
}

pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = user.id as u64;
    let users = &state.users;

    let cleanup_guard = CleanupGuard {
        user_id,
        users: users.clone(),
    };

    let rx = if let Some(tx) = users.get(&user_id) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        state.users.insert(user_id, tx);
        rx
    };
    let stream = BroadcastStream::new(rx).filter_map(|v| v.ok()).map(|v| {
        let name = match v.as_ref() {
            AppEvent::NewChat(_) => "NewChat",
            AppEvent::AddToChat(_) => "AddToChat",
            AppEvent::RemoveFromChat(_) => "RemoveFromChat",
            AppEvent::NewMessage(_) => "NewMessage",
            AppEvent::ChatNameUpdated(_) => "ChatNameUpdated",
        };
        let v = serde_json::to_string(&v).expect("Failed to serialize event");
        Ok(Event::default().data(v).event(name))
    });
    let guarded_stream = WithCleanup {
        stream,
        _guard: cleanup_guard,
    };
    Sse::new(guarded_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

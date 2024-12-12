pub(crate) mod chat;
pub(crate) mod file;
pub(crate) mod message;
pub(crate) mod user;
pub(crate) mod workspace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String, // extract ext from filename or mime type
    pub hash: String,
}

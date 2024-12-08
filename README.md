# Project Proposal: Rust-based Chat Application with axum Framework

## Overview

This project is a discord like chat application backend built with Rust, utilizing the **axum** framework for HTTP communication. It supports robust and scalable real-time messaging, including features such as **user authentication**, **one-to-one chats**, **group chats**, and **file sharing**. PostgreSQL serves as the database, with operations managed by the **sqlx** library. For real-time updates, **Server-Sent Events (SSE)** are used to deliver notifications to clients.

---

## Features

### Core Functionality
1. **User Authentication**
   - Secure login and signup using JSON Web Tokens (JWT).
   - Passwords are hashed with **argon2** before storage.

2. **Chat Messaging**
   - Support for one-on-one, group, and private/public channel chats.
   - Flexible chat types managed via PostgreSQL's custom enum types.

3. **File Sharing**
   - Upload and share files within chats.

4. **Real-Time Notifications**
   - Server-Sent Events (SSE) for message delivery and chat updates.

---

## System Architecture

The application is divided into three main components:
1. **API Server**
   - Manages user authentication, chat, and messaging APIs.
2. **Notification Server**
   - Pushes updates and new messages to clients using SSE.
3. **Database Layer**
   - PostgreSQL as the primary database, with Redis for pub/sub notifications.

---

## Database Schema

### 1. Users
The `users` table stores user details, including hashed passwords.
```sql
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    password VARCHAR(64) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

Rust Struct:
```rust
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

### 2. Chats
Chats can be one-on-one, group chats, or private/public channels.
```sql
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    type chat_type NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

Rust Struct:
```rust
struct Chat {
    id: usize,
    name: String,
    description: String,
    r#type: ChatType,
    users: Vec<usize>,
    notification: String,
    created_at: DateTime<Utc>,
}
```

### 3. Messages
Messages contain text content and optionally attached files.
```sql
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    images TEXT[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chat_id) REFERENCES chats(id),
    FOREIGN KEY (sender_id) REFERENCES users(id)
);
```

Rust Struct:
```rust
struct Message {
    id: usize,
    user_id: usize,
    chat_id: usize,
    content: String,
    files: Vec<String>,
    created_at: DateTime<Utc>,
}
```

---

## APIs

### 1. Authentication API
Endpoints for user registration and login:
- **POST `/api/signup`**: Register a new user.
- **POST `/api/signin`**: Authenticate a user and return a JWT token.

Example Request:
```json
POST /api/signin
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "password123"
}
```

### 2. Chat API
Endpoints for chat management:
- **POST `/api/chats`**: Create a new chat.
- **GET `/api/chats`**: Retrieve the list of available chats.
- **PATCH `/api/chats/:id`**: Update chat details (e.g., change type or members).

Example Request:
```json
POST /api/chats
Authorization: Bearer <token>
Content-Type: application/json

{
    "type": "single",
    "user_id": 123
}
```

### 3. Messaging API
Endpoints for sending and retrieving messages:
- **POST `/api/chats/:id/messages`**: Send a message to a chat.
- **GET `/api/chats/:id/messages`**: Retrieve chat messages with optional pagination.
- **POST `/api/upload`**: Upload a file for sharing.

Example Request:
```json
POST /api/chats/:id/messages
Authorization: Bearer <token>
Content-Type: application/json

{
    "content": "Hello, world!",
    "files": ["file1.png", "file2.jpg"]
}
```

### 4. Notification API


Notify Server:
- Tracks active user connections via a DashMap<u64, Sender>.
- Pushes real-time updates to connected clients.
pglistener:
- Listens to PostgreSQL notifications (via LISTEN).
- Responds to database triggers for changes in chats or messages.
- Triggers in PostgreSQL

Triggers are defined to notify the system of:
1. New Chats
2. Chat Updates (e.g., member additions/removals).
3. New Messages
4. Notification Flow


![image](./asset/notify_server.png)
Database changes trigger specific events (e.g., chat_member_changed).
The pglistener relays these notifications to the Notify Server.
The server processes these notifications and uses the DashMap to send updates to relevant users via SSE.
Mapping Users to Notifications

DashMap<u64, Sender> efficiently maps user_id to a Sender that streams events to the user over SSE.

Rust Struct for Notifications:
```rust
struct Notification {
    user_ids: HashSet<u64>,
    event: Event,
}

enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}
```

---

## Security

1. **Password Management**
   - Passwords are securely hashed using **argon2** before storing them in the database.

2. **Token-Based Authentication**
   - Authentication is handled using JWT tokens, which are validated for every request.

3. **Database Constraints**
   - Foreign key relationships ensure data consistency between `users`, `chats`, and `messages`.

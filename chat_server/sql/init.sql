CREATE TABLE IF NOT EXISTS users (
    id bigserial PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    --hashed argon2 password
    password VARCHAR(64) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users (email);

CREATE TYPE chat_type  AS ENUM(
    'single',
    'group',
    'private_channel',
    'public_ channel'  
)

CREATE TABLE IF NOT EXISTS chats (
    id bigserial PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    type chat_type NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);      

CREATE TABLE IF NOT EXISTS messages (
    id bigserial PRIMARY KEY,
    chat_id bigint NOT NULL,
    sender_id bigint NOT NULL,
    content text NOT NULL,
    images text[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    FOREIGN KEY (chat_id) REFERENCES chats(id),
    FOREIGN KEY (sender_id) REFERENCES users(id),
); 
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id);

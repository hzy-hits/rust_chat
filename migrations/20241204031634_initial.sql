CREATE TABLE IF NOT EXISTS users (
    id bigserial PRIMARY KEY,
    ws_id bigint NOT NULL,
    username VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL UNIQUE,
    --hashed argon2 password
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS workspaces(
  id bigserial PRIMARY KEY,
  name varchar(32) NOT NULL UNIQUE,
  owner_id bigint NOT NULL REFERENCES users(id),
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
BEGIN;
INSERT INTO users(id, ws_id, username, email, password_hash)
  VALUES (0, 0, 'super user', 'super@none.org', '');
INSERT INTO workspaces(id, name, owner_id)
  VALUES (0, 'none', 0);
COMMIT;
-- add foreign key constraint for ws_id for users
ALTER TABLE users
  ADD CONSTRAINT users_ws_id_fk FOREIGN KEY (ws_id) REFERENCES workspaces(id);


CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users (email);

CREATE TYPE chat_type  AS ENUM(
    'single',
    'group',
    'private_channel',
    'public_channel'
);

CREATE TABLE IF NOT EXISTS chats (
    id bigserial PRIMARY KEY,
    name VARCHAR(64) ,
    ws_id bigint NOT NULL REFERENCES workspaces(id),
    chat_type chat_type NOT NULL,
    members bigint[] NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (ws_id, name,members)
);

CREATE TABLE IF NOT EXISTS messages (
    id bigserial PRIMARY KEY,
    chat_id bigint NOT NULL REFERENCES chats(id),
    sender_id bigint NOT NULL REFERENCES users(id),
    content text NOT NULL,
    files text[] DEFAULT '{}',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_index ON messages(chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id);

-- create index for chat members
CREATE INDEX IF NOT EXISTS chat_members_index ON chats USING GIN(members);

-- Add migration script here



-- Workspace for the user
CREATE TABLE IF NOT EXISTS workspaces (
    id bigserial PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id bigint NOT NULL REFERENCES users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE users
    ADD COLUMN ws_id bigint REFERENCES 
workspaces(id);
ALTER TABLE chats
  ADD COLUMN ws_id bigint REFERENCES workspaces(id);

BEGIN;
INSERT INTO users(id, username, email, password_hash)
  VALUES (0, 'super user', 'super@none.org', '');
INSERT INTO workspaces(id, name, owner_id)
  VALUES (0, 'none', 0);
UPDATE
  users
SET
  ws_id = 0
WHERE
  id = 0;
COMMIT;

ALTER TABLE users
  ALTER COLUMN ws_id SET NOT NULL;

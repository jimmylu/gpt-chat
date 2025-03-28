-- Add migration script here
-- create user table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(100) NOT NULL UNIQUE,
    -- hashed argon2 password
    password_hash VARCHAR(255) NOT NULL,
    ws_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create index for users for email
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);

-- create chat type if not exists for postgres: single, group, private_channel, public_channel
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'chat_type') THEN
        CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');
    END IF;
END $$;


CREATE TABLE if not exists workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    owner_id BIGINT not NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


-- create table for chat
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    ws_id BIGINT NOT NULL REFERENCES workspaces(id),
    name VARCHAR(64),
    type chat_type NOT NULL,
    -- user id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- create message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    files TEXT[],
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS idx_messages_chat_id_created_at ON messages (chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages (sender_id, created_at DESC);


BEGIN;
-- insert default user
INSERT INTO users (id, fullname, email, ws_id, password_hash) VALUES (0, 'Default', 'default@default.com', 0, 'default');
-- insert default workspace
INSERT INTO workspaces (id, name, owner_id) VALUES (0, 'Default', 0);
UPDATE users SET ws_id = 0 WHERE id = 0;
INSERT INTO chats (id, name, type, members, ws_id) VALUES (0, 'Default', 'public_channel', ARRAY[0], 0);
COMMIT;

-- add foreign key constraint for ws_id for users
ALTER TABLE users
  ADD CONSTRAINT users_ws_id_fk FOREIGN KEY (ws_id) REFERENCES workspaces(id);

-- this file is used to initialize the  postgres database
-- create user table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(100) NOT NULL UNIQUE,
    -- hashed argon2 password
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- create index for users for email
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE IF NOT EXISTS chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- create table for chat
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    type chat_type NOT NULL,
    -- user id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- create message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP

    FOREIGN KEY (chat_id) REFERENCES chats(id),
    FOREIGN KEY (sender_id) REFERENCES users(id)

);

-- create index for messages for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS idx_messages_chat_id_created_at ON messages (chat_id, created_at DESC);

-- create index for messages for sender_id
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages (sender_id);

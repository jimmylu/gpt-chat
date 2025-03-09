-- Add migration script here
-- workspace for users
CREATE TABLE if not exists workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    owner_id BIGINT not NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- insert default workspace
INSERT INTO workspaces (id, name, owner_id) VALUES (0, 'Default', 0);

-- alter users table  to add workspace_id
ALTER TABLE users
    ADD COLUMN ws_id BIGINT REFERENCES workspaces(id) not null default 0;

-- alter chats table to add workspace_id
ALTER TABLE chats
    ADD COLUMN ws_id BIGINT REFERENCES workspaces(id);

-- create default workspace

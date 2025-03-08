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
INSERT INTO workspaces (name, owner_id) VALUES ('Default', 1);

-- alter users table  to add workspace_id
ALTER TABLE users
    ADD COLUMN ws_id BIGINT REFERENCES workspaces(id) not null default 1;

-- create default workspace

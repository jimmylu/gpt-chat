-- Add migration script here
-- workspace for users


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

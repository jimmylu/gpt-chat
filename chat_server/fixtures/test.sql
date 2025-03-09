-- insert 3 workspaces
insert into workspaces(name, owner_id)
VALUES ('Test ws', 0),
('Test ws 2', 0);

-- insert 5 users
insert into users(email, password_hash, fullname, ws_id)
VALUES ('test@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User', 1),
('test2@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 2', 1),
('test3@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 3', 2),
('test4@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 4', 2),
('test5@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 5', 2),
('test6@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 6', 2),
('test7@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 7', 2),
('test8@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 8', 2),
('test9@yahoo.com', '$argon2id$v=19$m=19456,t=2,p=1$XS5QS+jiOarheORcKR6K5g$8Of79bSnJ5LFbcfxT/xt7iXRuW5p5Lu7PR+ctGX/lzQ', 'Test User 9', 2);

-- insert 4 chats
-- insert public/private channel
insert into chats(name, ws_id, type, members)
VALUES ('Test Chat', 1, 'private_channel', ARRAY[1, 2]),
('Test Chat 2', 2, 'private_channel', ARRAY[3, 4, 5]),
('Test Chat 3', 1, 'private_channel', ARRAY[1, 2, 6]),
('Test Chat 4', 2, 'private_channel', ARRAY[3, 4, 5, 6]);

-- insert group chat
insert into chats(name, ws_id, type, members)
VALUES ('Test Chat 3', 1, 'group', ARRAY[1, 2]),
('Test Chat 4', 2, 'group', ARRAY[3, 4, 5]);

-- insert unnamed group chat
insert into chats(ws_id, type, members)
VALUES (1, 'group', ARRAY[1, 2]),
(2, 'group', ARRAY[3, 4, 5]);


CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    login TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    salt TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS private_messages (
    id SERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    key_index INT NOT NULL,
    receiver_id INT NOT NULL REFERENCES users ON DELETE CASCADE,
    sender_id INT NOT NULL REFERENCES users ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS chats (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id SERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    key_index INT NOT NULL,
    chat_id INT NOT NULL REFERENCES chats ON DELETE CASCADE,
    sender_id INT NOT NULL REFERENCES users ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS chat_members (
    id SERIAL PRIMARY KEY,
    chat_id INT NOT NULL REFERENCES chats ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES users ON DELETE CASCADE
);


-- TRIGGERS

CREATE OR REPLACE FUNCTION notify_new_private_message()
    RETURNS TRIGGER
AS 
$$
DECLARE
    user_login TEXT;
BEGIN
    user_login := (SELECT login FROM users WHERE id = NEW.sender_id)::text;
    PERFORM pg_notify('new_private_message', json_build_object('message', row_to_json(NEW), 'login', user_login)::text);
    RETURN NULL;
END;
$$
LANGUAGE PLPGSQL;

CREATE TRIGGER new_private_message
    AFTER INSERT
    ON private_messages
    FOR EACH ROW
    EXECUTE PROCEDURE notify_new_private_message();


CREATE OR REPLACE FUNCTION notify_new_chat_message()
    RETURNS TRIGGER
AS 
$$
DECLARE
    members_ids INT ARRAY;
    user_login TEXT;
BEGIN
    user_login := (SELECT login FROM users WHERE id = NEW.sender_id)::text;
    members_ids := (SELECT get_chat_members_by_id(NEW.chat_id));
    PERFORM pg_notify('new_chat_message', json_build_object('message', row_to_json(NEW), 'members_ids', members_ids, 'login', user_login)::text);
    RETURN NULL;
END;
$$
LANGUAGE PLPGSQL;

CREATE TRIGGER new_chat_message
    AFTER INSERT
    ON chat_messages
    FOR EACH ROW
    EXECUTE PROCEDURE notify_new_chat_message();

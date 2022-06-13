DROP TABLE users CASCADE;
DROP TABLE private_messages CASCADE;
DROP TABLE chats CASCADE;
DROP TABLE chat_messages CASCADE;
DROP TABLE chat_members CASCADE;

DROP FUNCTION IF EXISTS notify_new_private_message();
DROP FUNCTION IF EXISTS notify_new_chat_message();

DROP TRIGGER IF EXISTS new_private_message ON private_messages;
DROP TRIGGER IF EXISTS new_chat_message ON chat_messages;
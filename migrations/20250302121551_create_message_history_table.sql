-- Up
CREATE TABLE IF NOT EXISTS message_history (
    id INTEGER PRIMARY KEY,
    user_id INT NOT NULL,
    messages TEXT,
    CONSTRAINT messages_json CHECK (json_valid(messages))
);
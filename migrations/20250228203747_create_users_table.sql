-- Up
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    chat_id INT,
    username TEXT NULL,
    first_name TEXT NULL,
    last_name TEXT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
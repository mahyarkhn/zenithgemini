-- Up
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY,
    chat_id INT,
    sender_id INT,
    message_id INT,
    content TEXT NULL,
    response TEXT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
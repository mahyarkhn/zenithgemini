-- Up
CREATE TABLE IF NOT EXISTS users (
    id DECIMAL PRIMARY KEY,
    username TEXT NULL,
    first_name TEXT NULL,
    last_name TEXT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Down
DROP TABLE users;
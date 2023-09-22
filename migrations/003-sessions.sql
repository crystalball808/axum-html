CREATE TABLE IF NOT EXISTS sessions (
    id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) references users(id)
);

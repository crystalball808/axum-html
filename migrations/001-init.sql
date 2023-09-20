CREATE TABLE migration (
    last_migration_name VARCHAR(255)
);

INSERT INTO migration (
    last_migration_name
) VALUES ('001');

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);


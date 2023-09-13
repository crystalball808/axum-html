CREATE TABLE migration (
    last_migration_name VARCHAR(255),
);

INSERT INTO migration (
    last_migration_name,
) VALUES ( '001.init' );

CREATE TABLE users (
    id UUID NT NULL PRIMARY KEY,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
);


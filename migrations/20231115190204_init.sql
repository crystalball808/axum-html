CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT,
  password TEXT,
);

CREATE TABLE sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

INSERT INTO users (name) VALUES ('Tempo');
INSERT INTO users (name) VALUES ('Solomon');

CREATE TABLE posts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  body TEXT,
  author_id INTEGER,
  FOREIGN KEY (author_id) REFERENCES users(id)
);

INSERT INTO posts (body, author_id) VALUES ('This is my first post', 1);
INSERT INTO posts (body, author_id) VALUES ('This is my second post', 1);

INSERT INTO posts (body, author_id) VALUES ('Hi this is Solomon', 2);
INSERT INTO posts (body, author_id) VALUES ('I love music', 2);

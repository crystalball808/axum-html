CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT,
  name TEXT,
  password TEXT
);

CREATE TABLE sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

INSERT INTO users (email, name, password) VALUES ('tempo@tempo.com', 'Tempo', 'qwe');
INSERT INTO users (email, name, password) VALUES ('solomon@tempo.com', 'Solomon', 'asd');

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

create table likes (
  id integer primary key autoincrement,
  user_id integer,
  post_id integer,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(post_id) REFERENCES posts(id)
);

insert into likes (user_id, post_id) values (1, 1);
insert into likes (user_id, post_id) values (2, 1);
insert into likes (user_id, post_id) values (2, 2);
insert into likes (user_id, post_id) values (1, 3);
insert into likes (user_id, post_id) values (2, 3);

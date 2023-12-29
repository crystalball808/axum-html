create table comments (
  id integer primary key autoincrement,
  post_id integer,
  author_id integer,
  body text,
  FOREIGN KEY(author_id) REFERENCES users(id),
  FOREIGN KEY(post_id) REFERENCES posts(id)
);

use anyhow::Result;
use sqlx::{FromRow, Row, SqlitePool};

#[derive(FromRow, Debug)]
pub struct Post {
    pub id: i32,
    pub body: String,
    pub author: String,
    pub comments_count: i32,
    pub likes_count: i32,
    pub liked: bool,
}

#[derive(FromRow, Debug)]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
}

pub async fn get_by_id(
    connection_pool: &SqlitePool,
    user_id: Option<i32>,
    post_id: i32,
) -> Result<Post> {
    let user_id = user_id.unwrap_or(0);

    let query = "
SELECT 
    p.id, 
    p.body, 
    u.name AS author, 
    COUNT(c.id) AS comments_count,
    COUNT(l.id) AS likes_count,
    CASE WHEN SUM(l.user_id = $1) > 0 THEN 1 ELSE 0 END AS liked
FROM 
    posts p
JOIN 
    users u ON u.id = p.author_id
LEFT JOIN 
    likes l ON l.post_id = p.id
LEFT JOIN
    comments c on c.post_id = p.id
WHERE 
    p.id = $2 -- Replace $2 with the ID of the post you want to fetch
GROUP BY 
    p.id, p.body, u.name;
";

    Ok(sqlx::query_as::<_, Post>(query)
        .bind(user_id)
        .bind(post_id)
        .fetch_one(connection_pool)
        .await?)
}

/// Get all posts
/// `user_id` to determine if user liked a post
pub async fn get_all(connection_pool: &SqlitePool, user_id: Option<i32>) -> Result<Vec<Post>> {
    let user_id = user_id.unwrap_or(0);

    let query = "
SELECT 
    p.id, 
    p.body, 
    u.name AS author, 
    COUNT(c.id) AS comments_count,
    COUNT(l.id) AS likes_count,
    CASE WHEN SUM(l.user_id = $1) > 0 THEN 1 ELSE 0 END AS liked
FROM 
    posts p
JOIN 
    users u ON u.id = p.author_id
LEFT JOIN 
    likes l ON l.post_id = p.id
LEFT JOIN
    comments c on c.post_id = p.id
GROUP BY 
    p.id, p.body, u.name;
";

    Ok(sqlx::query_as::<_, Post>(query)
        .bind(user_id)
        .fetch_all(connection_pool)
        .await?)
}

pub async fn create_post(connection_pool: &SqlitePool, author_id: i32, body: &str) -> Result<i32> {
    Ok(
        sqlx::query("insert into posts (author_id, body) values ($1, $2) returning id")
            .bind(author_id)
            .bind(body)
            .fetch_one(connection_pool)
            .await?
            .get(0),
    )
}

pub async fn like_post(connection_pool: &SqlitePool, user_id: i32, post_id: i32) -> Result<Post> {
    let query = "
-- Insert a like
INSERT INTO likes (user_id, post_id)
VALUES ($1, $2); -- Replace with the actual user_id and post_id

-- Retrieve the post with the same fields
SELECT 
    p.id, 
    p.body, 
    u.name AS author, 
    COUNT(l.id) AS likes_count,
    CASE WHEN SUM(l.user_id = $1) > 0 THEN 1 ELSE 0 END AS liked
FROM 
    posts p
JOIN 
    users u ON u.id = p.author_id
LEFT JOIN 
    likes l ON l.post_id = p.id
WHERE
    p.id = $2; -- Replace with the actual post_id
GROUP BY 
    p.id, p.body, u.name;
";

    Ok(sqlx::query_as::<_, Post>(query)
        .bind(user_id)
        .bind(post_id)
        .fetch_one(connection_pool)
        .await?)
}

pub async fn remove_like(connection_pool: &SqlitePool, user_id: i32, post_id: i32) -> Result<Post> {
    let query = "
-- Insert a like
DELETE FROM likes
    WHERE user_id = $1
      AND post_id = $2;

-- Retrieve the post with the same fields
SELECT 
    p.id, 
    p.body, 
    u.name AS author, 
    COUNT(l.id) AS likes_count,
    CASE WHEN SUM(l.user_id = $1) > 0 THEN 1 ELSE 0 END AS liked
FROM 
    posts p
JOIN 
    users u ON u.id = p.author_id
LEFT JOIN 
    likes l ON l.post_id = p.id
WHERE
    p.id = $2; -- Replace with the actual post_id
GROUP BY 
    p.id, p.body, u.name;
";

    Ok(sqlx::query_as::<_, Post>(query)
        .bind(user_id)
        .bind(post_id)
        .fetch_one(connection_pool)
        .await?)
}

#[derive(FromRow, Debug)]
pub struct Comment {
    pub body: String,
    pub author: String,
}

pub async fn comments(connection_pool: &SqlitePool, post_id: i32) -> Result<Vec<Comment>> {
    let query = "select body, u.name as author from comments c join users u on c.author_id = u.id where c.post_id = $1";

    Ok(sqlx::query_as::<_, Comment>(query)
        .bind(post_id)
        .fetch_all(connection_pool)
        .await?)
}

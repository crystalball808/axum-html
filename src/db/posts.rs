use anyhow::Result;
use sqlx::{FromRow, Row, SqlitePool};

#[derive(FromRow, Debug)]
pub struct Post {
    pub id: i32,
    pub body: String,
    pub author: String,
    pub likes_count: i32,
    pub liked: bool,
}

#[derive(FromRow, Debug)]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
}

pub async fn get_posts(connection_pool: &SqlitePool, user_id: Option<i32>) -> Result<Vec<Post>> {
    let user_id = user_id.unwrap_or(0);

    let query = "
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
    println!("Try to like a post");
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

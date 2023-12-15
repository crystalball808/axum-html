use anyhow::Result;
use sqlx::{FromRow, Row, SqlitePool};

#[derive(FromRow, Debug)]
pub struct Post {
    pub id: i32,
    pub body: String,
    pub author: String,
}

pub async fn get_posts(connection_pool: &SqlitePool) -> Result<Vec<Post>> {
    Ok(
        sqlx::query_as::<_, Post>("SELECT p.id, p.body, u.name as author FROM posts p join users u on u.id = p.author_id")
            .fetch_all(connection_pool)
            .await?,
    )
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

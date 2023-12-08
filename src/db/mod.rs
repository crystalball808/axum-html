use anyhow::Result;
use sqlx::{FromRow, Row, SqlitePool};

#[derive(FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
}

#[derive(FromRow, Debug)]
pub struct Session {
    user_id: i32,
}

#[derive(FromRow, Debug)]
pub struct Post {
    pub id: i32,
    pub body: String,
    pub author: String,
}

pub async fn init() -> Result<SqlitePool> {
    let database_url = std::env::var("DATABASE_URL")?;
    let connection_pool = SqlitePool::connect(&database_url).await?;

    sqlx::migrate!().run(&connection_pool).await?;

    Ok(connection_pool)
}

pub async fn check_session_id(connection_pool: &SqlitePool, session_id: i32) -> Result<bool> {
    let result = sqlx::query_as::<_, Session>("SELECT user_id FROM sessions WHERE id=$1")
        .bind(session_id)
        .fetch_optional(connection_pool)
        .await?;

    return Ok(result.is_some());
}

pub async fn get_user_from_session(
    connection_pool: &SqlitePool,
    session_id: i32,
) -> Result<Option<User>> {
    Ok(sqlx::query_as::<_, User>(
            "select u.id, u.name, u.email from users u join sessions s on u.id = s.user_id where s.id = $1"
        )
    .bind(session_id).fetch_optional(connection_pool).await?)
}

#[derive(FromRow, Debug)]
pub struct UserId {
    pub id: i32,
}
pub async fn get_user_id_from_login(
    connection_pool: &SqlitePool,
    email: &str,
    password: &str,
) -> Result<Option<i32>> {
    let result = sqlx::query_as::<_, UserId>("SELECT id FROM users WHERE email=$1 AND password=$2")
        .bind(email)
        .bind(password)
        .fetch_optional(connection_pool)
        .await?;

    Ok(match result {
        Some(user) => Some(user.id),
        None => None,
    })
}

pub async fn check_email_exists(connection_pool: &SqlitePool, email: &str) -> Result<bool> {
    let result = sqlx::query_as::<_, User>("SELECT id FROM users WHERE email=$1")
        .bind(email)
        .fetch_optional(connection_pool)
        .await?;

    Ok(result.is_some())
}

pub async fn create_user(
    connection_pool: &SqlitePool,
    email: &str,
    name: &str,
    password: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO users (email, name, password) VALUES ($1, $2, $3)")
        .bind(email)
        .bind(name)
        .bind(password)
        .execute(connection_pool)
        .await?;
    Ok(())
}

pub async fn create_session(connection_pool: &SqlitePool, user_id: i32) -> Result<i32> {
    Ok(
        sqlx::query("INSERT INTO sessions (user_id) VALUES ($1) RETURNING id")
            .bind(user_id)
            .fetch_one(connection_pool)
            .await?
            .get(0),
    )
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

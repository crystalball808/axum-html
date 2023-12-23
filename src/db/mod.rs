use std::str::FromStr;

use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, FromRow, Row, SqlitePool};

pub mod posts;

#[derive(FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
}

pub async fn init() -> Result<SqlitePool> {
    let database_url = std::env::var("DATABASE_URL")?;
    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    let connection_pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&connection_pool).await?;

    Ok(connection_pool)
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
    let result = sqlx::query!("SELECT id FROM users WHERE email=$1", email)
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

pub async fn delete_session_by_id(connection_pool: &SqlitePool, session_id: i32) -> Result<()> {
    sqlx::query("delete from sessions where id = $1")
        .bind(session_id)
        .execute(connection_pool)
        .await?;

    Ok(())
}

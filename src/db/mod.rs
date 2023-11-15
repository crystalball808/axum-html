use anyhow::Result;
use sqlx::SqlitePool;

pub struct User {
    id: i32,
    name: String,
}
pub struct Session {
    id: i32,
    user_id: i32,
}

pub struct Post {
    id: i32,
    body: String,
    author_id: i32,
}

pub async fn init() -> Result<SqlitePool> {
    let database_url = std::env::var("DATABASE_URL")?;
    let connection_pool = SqlitePool::connect(&database_url).await?;

    sqlx::migrate!().run(&connection_pool).await?;

    Ok(connection_pool)
}

pub(crate) fn check_session_id(connection_pool: &SqlitePool, session_id: i32) -> bool {
    todo!()
}

pub(crate) async fn get_user_id_from_login(connection_pool: &SqlitePool, email: &str, password: &str) -> Option<i32> {
    todo!()
}

use libsql_client::{Client, Config, Statement};
use std::{env::var, error::Error};

mod migrations;

// Add a flag to setup tables
pub async fn setup() -> Result<Client, Box<dyn Error>> {
    let turso_url = var("TURSO_URL")?;
    let turso_token = var("TURSO_TOKEN")?;
    let client = Client::from_config(Config {
        url: turso_url.as_str().try_into()?,
        auth_token: Some(turso_token),
    })
    .await?;

    let _ = migrations::setup_migrations(&client).await?;

    return Ok(client);
}

pub fn check_session_id(session_id: u32) -> bool {
    todo!()
}

pub async fn get_user_id_from_login(client: &Client, email: &str, password: &str) -> Option<u32> {
    // Maybe use libsql_client::Statement ???
    let response = client
        .execute(format!(
            "SELECT id FROM users WHERE email = {email} AND password = {password}"
        ))
        .await;
    dbg!(&response);

    todo!();
}

pub async fn check_email_exists(client: &Client, email: &str) -> Result<bool, Box<dyn Error>> {
    let statement = Statement::with_args("SELECT * FROM users WHERE email = ?", &[email]);
    let response = client.execute(statement).await?;

    return Ok(response.rows.len() > 0);
}

pub async fn create_user(
    client: &Client,
    first_name: &str,
    last_name: &str,
    email: &str,
    password: &str,
) -> Result<impl Send, Box<dyn Error + Send>> {
    let statement = Statement::with_args(
        "INSERT INTO users (first_name, last_name, email, password)
VALUES ('John', 'Doe', 'john.doe@example.com', 'password123');",
        &[first_name, last_name, email, password],
    );

    let _ = client.execute(statement).await?;

    Ok(())
}


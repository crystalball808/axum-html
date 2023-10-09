use libsql_client::{Client, Config};
use std::{env::var, error::Error, fmt::format};

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
    let response = client.execute(format!("SELECT id FROM users WHERE email = {email} AND password = {password}")).await;
    dbg!(&response);
    
    // TODO: remove it
    None
}

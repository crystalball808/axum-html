use libsql_client::{Client, Config};
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

    println!("Hello bitch");
    let _ = migrations::setup_migrations(&client)?;

    return Ok(client);
}

use libsql_client::Client;
use std::{fs, io};

pub fn setup_migrations(client: &Client) -> io::Result<()> {
    let files = fs::read_dir("./migrations")?;

    for file in files {
        let file = file?;
    }

    Ok(())
}

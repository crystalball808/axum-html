use libsql_client::Client;
use std::{fs, io};

pub fn setup_migrations(client: &Client) -> io::Result<()> {
    let files = fs::read_dir("migrations")?;
    dbg!(&files);

    for file in files {
        let file = file?;
        let path = file.path();

        if path.is_file() {
            let sql_query = fs::read_to_string(&path);

            println!(
                "file name: {}\n query:\n{}",
                path.to_str().unwrap(),
                sql_query.unwrap()
            );
        }
    }

    Ok(())
}

use libsql_client::Client;
use std::{error::Error, fs};

async fn get_last_migration_number(client: &Client) -> Result<u32, Box<dyn Error>> {
    let query_result = client
        .execute("SELECT last_migration_name FROM migration")
        .await?;
    let row = &query_result.rows[0];
    let last_migration_name: &str = row.try_get(0)?;

    let last_migration_number: u32 = last_migration_name.parse()?;

    return Ok(last_migration_number);
}

async fn set_last_migration_number(
    client: &Client,
    last_migration_number: &str,
) -> Result<(), Box<dyn Error>> {
    client
        .execute(format!(
            "INSERT INTO migration (last_migration_name) VALUES ('{}');",
            last_migration_number
        ))
        .await?;

    Ok(())
}

pub async fn setup_migrations(client: &Client) -> Result<(), Box<dyn Error>> {
    let files = fs::read_dir("migrations")?;

    let last_migration_number = get_last_migration_number(client).await.unwrap_or(0);

    for file in files {
        let file = file?;
        let path = file.path();

        if path.is_file() {
            let migration_name = path.to_str().unwrap().split("/").nth(1).unwrap();
            let migration_name = migration_name.split("-").nth(0).unwrap();

            let migration_number: u32 = migration_name.parse()?;

            if last_migration_number < migration_number {
                // execute migration
                let tx = client.transaction().await.unwrap();

                let sql_query = fs::read_to_string(&path)?;
                let sql_statements = sql_query.split(";");

                for sql_statement in sql_statements {
                    let sql_statement = sql_statement.trim();

                    if sql_statement.len() > 0 {
                        tx.execute(sql_statement).await?;
                    }
                }

                tx.commit().await?;
                set_last_migration_number(&client, migration_name).await?;
            }
        }
    }

    Ok(())
}

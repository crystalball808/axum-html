use anyhow::Result;
use axum::{routing::get_service, Extension};
use tower_http::services::ServeFile;

mod db;
mod helpers;
mod routes;
mod utils;

use routes::setup_router;

#[tokio::main]
async fn main() -> Result<()> {
    match utils::generate_styles() {
        Ok(generate_styles_output) => println!("Generate styles output:{generate_styles_output}"),
        Err(error) => panic!("Failed to generate styles: {error}"),
    }
    dotenv::dotenv().ok();

    let connection_pool = db::init().await?;

    let app = setup_router()
        .route(
            "/static/styles.css",
            get_service(ServeFile::new("static/tailwind-generated.css")),
        )
        .layer(Extension(connection_pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

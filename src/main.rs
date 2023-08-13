use axum::{routing::get, Router, response::Html};

async fn home() -> Html<&'static str> {
    Html("<h1>Hello world!</h1>")
}

#[tokio::main]
async fn main() {

    let app = Router::new().route("/", get(home));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

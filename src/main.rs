use askama::Template;
use axum::{
    extract::{self, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service, post},
    Form, Router,
};
use hyper::HeaderMap;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::signal;
use tower_http::services::ServeFile;
use uuid::Uuid;

#[derive(Clone)]
struct Todo {
    id: String,
    label: String,
    completed: bool,
}
#[derive(Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        todos: Arc::new(Mutex::new(vec![Todo {
            id: Uuid::new_v4().to_string(),
            label: "Make a super app".to_owned(),
            completed: false,
        }])),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/todos", get(todos))
        .route("/todos", post(create_todo))
        .route("/greet/:name", get(greet))
        .route("/clicked", post(clicked))
        .route(
            "/static/styles.css",
            get_service(ServeFile::new("static/tailwind-generated.css")),
        )
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    let html = include_str!("../templates/index.html");
    Html(html)
}
async fn greet(extract::Path(name): extract::Path<String>) -> impl IntoResponse {
    let template = HelloTemplate { name };
    HtmlTemplate(template)
}
async fn todos(State(state): State<AppState>) -> impl IntoResponse {
    let todos = state.todos.lock().expect("Failed to lock the state");
    let template = TodosTemplate {
        todos: todos.to_vec(),
    };
    HtmlTemplate(template)
}

async fn clicked() -> Html<&'static str> {
    Html("<p>Wow you are so cool!</p>")
}

#[derive(Deserialize)]
struct TodoForm {
    todo: String,
}

async fn create_todo(State(state): State<AppState>, Form(todo_form): Form<TodoForm>) -> impl IntoResponse {
    println!("New todo: {}", todo_form.todo);
    let new_todo = Todo {
        id: Uuid::new_v4().to_string(),
        label: todo_form.todo,
        completed: false,
    };

    let mut todos = state.todos.lock().expect("Create todo: failed to lock todos");
    todos.push(new_todo);

    let mut headers = HeaderMap::new();
    headers.insert("HX-Trigger", "todoCreated".parse().unwrap());

    (headers, StatusCode::CREATED)
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

#[derive(Template)]
#[template(path = "todos.html")]
struct TodosTemplate {
    todos: Vec<Todo>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

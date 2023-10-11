use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    Form, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use hyper::HeaderMap;
use libsql_client::Client;
use serde::Deserialize;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tokio::signal;
use tower_http::services::ServeFile;
use uuid::Uuid;
mod routes;
use dotenv::dotenv;
use routes::todos::{get_todos, Todo};

#[derive(Clone)]
pub struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    db_client: Arc<Client>,
}

mod db;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_client = match db::setup().await {
        Ok(client) => client,
        Err(error) => {
            println!("DB client setup failed\nError: {}", error);
            std::process::exit(1);
        }
    };

    let state = AppState {
        todos: Arc::new(Mutex::new(vec![Todo {
            id: Uuid::new_v4().to_string(),
            label: "Make a super app".to_owned(),
            completed: false,
        }])),
        db_client: Arc::new(db_client),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login_page))
        .route("/login", post(login))
        .route("/login-form", get(login_form))
        .route("/register", post(register))
        .route("/register-form", get(register_form))
        .route("/todos", get(get_todos))
        .route("/todos", post(create_todo))
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

async fn index(jar: CookieJar, state: State<AppState>) -> Response {
    if let Some(session_id) = jar.get("sesstion_id") {
        let session_id: u32 = session_id.to_string().parse().unwrap();
        if db::check_session_id(session_id) {
            let todos = state.todos.lock().expect("Failed to lock the state");
            let template = IndexTemplate {
                todos: todos.to_vec(),
            };
            return Html(template.to_string()).into_response();
        }
    }

    return Redirect::permanent("/login").into_response();
}

async fn login_page() -> Response {
    let template = LoginPageTemplate {};
    return Html(template.to_string()).into_response();
}
async fn login_form() -> Response {
    return Html(fs::read_to_string("templates/login-form.html").unwrap()).into_response();
}
async fn register_form() -> Response {
    return Html(fs::read_to_string("templates/register-form.html").unwrap()).into_response();
}

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}
async fn login(
    State(state): State<AppState>,
    Form(login_form): Form<LoginForm>,
) -> impl IntoResponse {
    let db_client = Arc::clone(&state.db_client);
    let user_id =
        db::get_user_id_from_login(&db_client, &login_form.email, &login_form.password).await;

    match user_id {
        Some(user_id) => {}
        None => {}
    }
}

#[derive(Deserialize, Debug)]
struct RegisterForm {
    email: String,
    first_name: String,
    last_name: String,
    password: String,
}
async fn register(
    State(state): State<AppState>,
    Form(register_form): Form<RegisterForm>,
) -> Response {
    let db_client = Arc::clone(&state.db_client);
    dbg!(&register_form);
    let email_exists = db::check_email_exists(&db_client, &register_form.email).await;

    if email_exists {
        return StatusCode::BAD_REQUEST.into_response();
    } else {
        todo!();
    }
}

#[derive(Deserialize)]
struct TodoForm {
    todo: String,
}

async fn create_todo(
    State(state): State<AppState>,
    Form(todo_form): Form<TodoForm>,
) -> impl IntoResponse {
    println!("New todo: {}", todo_form.todo);
    let new_todo = Todo {
        id: Uuid::new_v4().to_string(),
        label: todo_form.todo,
        completed: false,
    };

    let mut todos = state
        .todos
        .lock()
        .expect("Create todo: failed to lock todos");
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
#[template(path = "index.html")]
struct IndexTemplate {
    todos: Vec<Todo>,
}
#[derive(Template)]
#[template(path = "login.html")]
struct LoginPageTemplate {}

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

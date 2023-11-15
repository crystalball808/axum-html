use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    Extension, Form, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tokio::signal;
use tower_http::services::ServeFile;
use uuid::Uuid;
mod routes;
use routes::todos::{get_todos, Todo};
use anyhow::Result;

#[derive(Clone)]
pub struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
}

mod db;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let connection_pool = db::init().await?;

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
        .layer(Extension(connection_pool));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn index(jar: CookieJar, state: State<AppState>) -> Response {
    if let Some(session_id) = jar.get("sesstion_id") {
        let session_id: i32 = session_id.to_string().parse().unwrap();
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
    Extension(connection_pool): Extension<SqlitePool>,
    Form(login_form): Form<LoginForm>,
) -> impl IntoResponse {
    let user_id =
        db::get_user_id_from_login(&connection_pool, &login_form.email, &login_form.password).await;

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
    Extension(db_client): Extension<Arc<Client>>,
    Form(register_form): Form<RegisterForm>,
) -> Response {
    let email_exists_result = db::check_email_exists(&db_client, &register_form.email).await;

    if let Ok(email_exists) = email_exists_result {
        if email_exists {
            return (StatusCode::BAD_REQUEST).into_response();
        } else {
            if db::create_user(
                &db_client,
                &register_form.first_name,
                &register_form.last_name,
                &register_form.email,
                &register_form.password,
            )
            .await
            .is_ok()
            {
                return (Redirect::to("/login")).into_response();
            } else {
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
        }
    } else {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
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

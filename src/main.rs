use anyhow::Result;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    Extension, Form, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use db::Post;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fs;
use tokio::signal;
use tower_http::services::ServeFile;

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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<Post>,
}

async fn index(jar: CookieJar, Extension(connection_pool): Extension<SqlitePool>) -> Response {
    println!("index get handler");
    if let Some(session_id) = jar.get("session_id") {
        let session_id: i32 = session_id.value().parse::<i32>().unwrap();

        match db::check_session_id(&connection_pool, session_id).await {
            Ok(present) => {
                if !present {
                    return Redirect::temporary("/login").into_response();
                }

                if let Ok(posts) = db::get_posts(&connection_pool).await {
                    let template = IndexTemplate { posts };
                    return Html(template.to_string()).into_response();
                }
            }
            Err(error) => {
                println!("{error}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    return Redirect::temporary("/login").into_response();
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
    jar: CookieJar,
    Form(login_form): Form<LoginForm>,
) -> impl IntoResponse {
    let user_id =
        db::get_user_id_from_login(&connection_pool, &login_form.email, &login_form.password).await;

    match user_id {
        Ok(user_id) => {
            if let Some(user_id) = user_id {
                dbg!(user_id);
                if let Ok(session_id) = db::create_session(&connection_pool, user_id).await {
                    dbg!(session_id);
                    return (
                        jar.add(Cookie::new("session_id", session_id.to_string())),
                        Redirect::temporary("/"),
                    )
                        .into_response();
                }
            }
        }
        Err(error) => {
            println!("{error}");
        }
    }
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[derive(Deserialize, Debug)]
struct RegisterForm {
    name: String,
    email: String,
    password: String,
}

async fn register(
    Extension(connection_pool): Extension<SqlitePool>,
    Form(register_form): Form<RegisterForm>,
) -> Response {
    let email_exists_result = db::check_email_exists(&connection_pool, &register_form.email).await;

    if let Ok(email_exists) = email_exists_result {
        if email_exists {
            return (StatusCode::BAD_REQUEST).into_response();
        } else {
            if db::create_user(
                &connection_pool,
                &register_form.email,
                &register_form.name,
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

// #[derive(Deserialize)]
// struct TodoForm {
//     todo: String,
// }

// async fn create_todo(Form(todo_form): Form<TodoForm>) -> impl IntoResponse {
//     println!("New todo: {}", todo_form.todo);
//     let new_todo = Todo {
//         id: Uuid::new_v4().to_string(),
//         label: todo_form.todo,
//         completed: false,
//     };
//
//     let mut headers = HeaderMap::new();
//     headers.insert("HX-Trigger", "todoCreated".parse().unwrap());
//
//     (headers, StatusCode::CREATED)
// }

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
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

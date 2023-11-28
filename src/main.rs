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
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fs;
use tower_http::services::ServeFile;

mod db;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let generate_styles_output = utils::generate_styles();
    println!("Generate styles output:{generate_styles_output}");
    dotenv::dotenv().ok();

    let connection_pool = db::init().await?;

    let app = Router::new()
        .route("/", get(index))
        .route("/login", post(login))
        .route("/login-form", get(login_form))
        .route("/register", post(register))
        .route("/register-form", get(register_form))
        .route(
            "/static/styles.css",
            get_service(ServeFile::new("static/tailwind-generated.css")),
        )
        .layer(Extension(connection_pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    user_name: Option<&'a str>,
    posts: Vec<Post>,
}

async fn index(jar: CookieJar, Extension(connection_pool): Extension<SqlitePool>) -> Response {
    let user_name: Option<String> = match jar.get("session_id") {
        Some(session_id) => {
            let session_id: i32 = match session_id.value().parse() {
                Ok(session_id) => session_id,
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };

            match db::get_user_name_from_session_id(&connection_pool, session_id).await {
                Ok(user_name) => user_name,
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        None => None,
    };

    let posts = match db::get_posts(&connection_pool).await {
        Ok(posts) => posts,
        Err(error) => {
            dbg!(error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let template = IndexTemplate {
        user_name: user_name.as_deref(),
        posts,
    };

    Html(template.to_string()).into_response()
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
                    let mut headers = HeaderMap::new();
                    headers.insert("HX-Refresh", "true".parse().unwrap());
                    return (
                        jar.add(Cookie::new("session_id", session_id.to_string())),
                        headers,
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

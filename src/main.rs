use anyhow::Result;
use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, get_service, post},
    Extension, Form, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use db::posts::Post;
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fs;
use tower_http::services::ServeFile;

use crate::db::User;

mod db;
mod utils;

const SESSION_ID_COOKIE_KEY: &str = "session_id";
fn get_session_id(jar: &CookieJar) -> Option<i32> {
    let session_id_cookie = jar.get(SESSION_ID_COOKIE_KEY);

    if let Some(session_id) = session_id_cookie {
        let parsed_session_id = session_id.value().parse::<i32>();

        if let Ok(session_id) = parsed_session_id {
            return Some(session_id);
        }
    }

    None
}

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
        .route("/posts", get(get_posts))
        .route("/posts", post(create_post))
        .route("/likes/:post_id", post(like_post))
        .route("/likes/:post_id", delete(unlike_post))
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
#[template(path = "like-button.html")]
struct LikeButtonTemplate {
    post: Post,
}
async fn like_post(
    jar: CookieJar,
    Path(post_id): Path<i32>,
    Extension(connection_pool): Extension<SqlitePool>,
) -> Response {
    let session_id = get_session_id(&jar);
    if session_id.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let session_id = session_id.unwrap();

    let user = match db::get_user_from_session(&connection_pool, session_id).await {
        Ok(user) => user,
        Err(error) => {
            dbg!(error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user = user.unwrap();

    match db::posts::like_post(&connection_pool, user.id, post_id).await {
        Ok(post) => {
            let like_button_template = LikeButtonTemplate { post };
            Html(like_button_template.to_string()).into_response()
        },
        Err(error) => {
            dbg!(&error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
async fn unlike_post(jar: CookieJar, Path(post_id): Path<i32>, Extension(connection_pool): Extension<SqlitePool>) -> Response {
    let session_id = get_session_id(&jar);
    if session_id.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let session_id = session_id.unwrap();

    let user = match db::get_user_from_session(&connection_pool, session_id).await {
        Ok(user) => user,
        Err(error) => {
            dbg!(error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user = user.unwrap();

    match db::posts::remove_like(&connection_pool, user.id, post_id).await {
        Ok(post) => {
            let like_button_template = LikeButtonTemplate { post };
            Html(like_button_template.to_string()).into_response()
        },
        Err(error) => {
            dbg!(&error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    user_name: Option<&'a str>,
    posts: Vec<Post>,
}

async fn index(jar: CookieJar, Extension(connection_pool): Extension<SqlitePool>) -> Response {
    let user: Option<User> = match jar.get(SESSION_ID_COOKIE_KEY) {
        Some(session_id) => {
            let session_id: i32 = match session_id.value().parse() {
                Ok(session_id) => session_id,
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };

            match db::get_user_from_session(&connection_pool, session_id).await {
                Ok(user) => user,
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        None => None,
    };

    let user_id = match &user {
        Some(user) => Some(user.id),
        None => None,
    };
    let posts = match db::posts::get_posts(&connection_pool, user_id).await {
        Ok(posts) => posts,
        Err(error) => {
            dbg!(error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    dbg!(&posts);

    let user_name = match user {
        Some(user) => Some(user.name),
        None => None,
    };
    let template = IndexTemplate {
        user_name: user_name.as_deref(),
        posts,
    };

    let html_response = template.to_string();

    if user_name.is_some() {
        Html(html_response).into_response()
    } else {
        (jar.remove(SESSION_ID_COOKIE_KEY), Html(html_response)).into_response()
    }
}

async fn login_form() -> Response {
    return Html(fs::read_to_string("templates/login-form.html").unwrap()).into_response();
}
#[derive(Template)]
#[template(path = "register-form.html")]
struct RegisterFormTemplate<'a> {
    user_name: Option<&'a str>,
}
async fn register_form() -> Response {
    let template = RegisterFormTemplate {
        user_name: Some(""),
    };
    return Html(template.to_string()).into_response();
}

#[derive(Template)]
#[template(path = "posts.html")]
struct PostsTemplate {
    posts: Vec<Post>,
}
async fn get_posts(jar: CookieJar, Extension(connection_pool): Extension<SqlitePool>) -> Response {
    let user: Option<User> = match jar.get(SESSION_ID_COOKIE_KEY) {
        Some(session_id) => {
            let session_id: i32 = match session_id.value().parse() {
                Ok(session_id) => session_id,
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };

            match db::get_user_from_session(&connection_pool, session_id).await {
                Ok(user) => user,
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        None => None,
    };

    let user_id = match user {
        Some(user) => Some(user.id),
        None => None,
    };
    let posts = match db::posts::get_posts(&connection_pool, user_id).await {
        Ok(posts) => posts,
        Err(error) => {
            println!("{error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let template = PostsTemplate { posts };

    Html(template.to_string()).into_response()
}

#[derive(Deserialize)]
struct PostForm {
    body: String,
}
async fn create_post(
    jar: CookieJar,
    Extension(connection_pool): Extension<SqlitePool>,
    Form(post_form): Form<PostForm>,
) -> Response {
    let user_id: i32 = match jar.get(SESSION_ID_COOKIE_KEY) {
        Some(session_id) => {
            let session_id: i32 = match session_id.value().parse() {
                Ok(session_id) => session_id,
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };

            match db::get_user_from_session(&connection_pool, session_id).await {
                Ok(user) => match user {
                    Some(user) => user.id,
                    None => {
                        return (
                            jar.remove("session_id"),
                            StatusCode::NETWORK_AUTHENTICATION_REQUIRED,
                        )
                            .into_response();
                    }
                },
                Err(error) => {
                    dbg!(error);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        None => {
            return StatusCode::NETWORK_AUTHENTICATION_REQUIRED.into_response();
        }
    };

    match db::posts::create_post(&connection_pool, user_id, &post_form.body).await {
        Ok(_) => {
            println!("Created a post: {}", post_form.body);
            let mut headers = HeaderMap::new();
            headers.insert("HX-Trigger", "postCreated".parse().unwrap());

            return (headers, StatusCode::CREATED).into_response();
        }
        Err(error) => {
            println!("{error}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
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
                let mut headers = HeaderMap::new();
                headers.insert("HX-Redirect", "/".parse().unwrap());
                return headers.into_response();
            } else {
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
        }
    } else {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }
}

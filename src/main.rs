use anyhow::Result;
use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, get_service, post},
    Extension, Form,
};
use axum_extra::extract::cookie::CookieJar;
use db::posts::Post;
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_http::services::ServeFile;

use db::User;
use helpers::get_session_id;
use routes::setup_router;

mod db;
mod helpers;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    match utils::generate_styles() {
        Ok(generate_styles_output) => println!("Generate styles output:{generate_styles_output}"),
        Err(error) => panic!("Failed to generate styles: {error}"),
    }
    dotenv::dotenv().ok();

    let connection_pool = db::init().await?;

    let app = setup_router()
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
        }
        Err(error) => {
            dbg!(&error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
async fn unlike_post(
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

    match db::posts::remove_like(&connection_pool, user.id, post_id).await {
        Ok(post) => {
            let like_button_template = LikeButtonTemplate { post };
            Html(like_button_template.to_string()).into_response()
        }
        Err(error) => {
            dbg!(&error);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "posts.html")]
struct PostsTemplate {
    posts: Vec<Post>,
}
async fn get_posts(jar: CookieJar, Extension(connection_pool): Extension<SqlitePool>) -> Response {
    let user: Option<User> = match get_session_id(&jar) {
        Some(session_id) => match db::get_user_from_session(&connection_pool, session_id).await {
            Ok(user) => user,
            Err(error) => {
                dbg!(error);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
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
    let user_id: i32 = match get_session_id(&jar) {
        Some(session_id) => match db::get_user_from_session(&connection_pool, session_id).await {
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
        },
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

use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Extension, Form, Router,
};
use axum_extra::extract::cookie::CookieJar;
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    db::{self, posts::Post, User},
    helpers::get_session_id,
};

pub fn setup_posts_router() -> Router {
    Router::new()
        .route("/posts", get(get_posts))
        .route("/posts/:post_id", get(get_one_post))
        .route("/posts", post(create_post))
        .route("/likes/:post_id", post(like_post))
        .route("/likes/:post_id", delete(unlike_post))
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate<'a> {
    post: Post,
    user_name: Option<&'a str>,
}
async fn get_one_post(
    jar: CookieJar,
    Path(post_id): Path<i32>,
    Extension(connection_pool): Extension<SqlitePool>,
) -> Response {
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
    let user_id = user.as_ref().map(|u| u.id);

    let post = match db::posts::get_by_id(&connection_pool, user_id, post_id).await {
        Ok(post) => post,
        Err(error) => {
            dbg!(error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let user_name = user.map(|u| u.name);

    let post_template = PostTemplate {
        post,
        user_name: user_name.as_deref(),
    };

    Html(post_template.to_string()).into_response()
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

    let user_id = user.map(|u| u.id);

    let posts = match db::posts::get_all(&connection_pool, user_id).await {
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

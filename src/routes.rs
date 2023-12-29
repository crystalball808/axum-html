mod auth;
mod posts;
use askama::Template;
use auth::setup_auth_router;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use axum_extra::{extract::CookieJar, response::Html};
use posts::setup_posts_router;
use sqlx::SqlitePool;

use crate::{
    db::{self, posts::Post, User},
    helpers::{get_session_id, SESSION_ID_COOKIE_KEY},
};

pub fn setup_router() -> Router {
    Router::new()
        .route("/", get(index))
        .merge(setup_auth_router())
        .merge(setup_posts_router())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    user_name: Option<&'a str>,
    posts: Vec<Post>,
}
async fn index(jar: CookieJar, Extension(connection_pool): Extension<SqlitePool>) -> Response {
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
    let posts = match db::posts::get_all(&connection_pool, user_id).await {
        Ok(posts) => posts,
        Err(error) => {
            dbg!(error);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let user_name = user.map(|u| u.name);
    let template = IndexTemplate {
        user_name: user_name.as_deref(),
        posts,
    };

    let html_response = template.to_string();

    if user_name.is_some() {
        axum::response::Html(html_response).into_response()
    } else {
        (jar.remove(SESSION_ID_COOKIE_KEY), Html(html_response)).into_response()
    }
}

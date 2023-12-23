use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Extension, Form, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fs;

use crate::{
    db,
    helpers::{self, SESSION_ID_COOKIE_KEY},
};

pub fn setup_auth_router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/login-form", get(login_form))
        .route("/register", post(register))
        .route("/register-form", get(register_form))
        .route("/logout", post(logout))
}

async fn logout(Extension(connection_pool): Extension<SqlitePool>, jar: CookieJar) -> Response {
    let session_id = helpers::get_session_id(&jar);
    if session_id.is_none() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let session_id = session_id.unwrap();

    db::delete_session_by_id(&connection_pool, session_id).await;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Refresh", "true".parse().unwrap());

    (headers, jar.remove(SESSION_ID_COOKIE_KEY)).into_response()
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

async fn login_form() -> Response {
    return Html(fs::read_to_string("templates/login-form.html").unwrap()).into_response();
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

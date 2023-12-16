use axum_extra::extract::CookieJar;

pub const SESSION_ID_COOKIE_KEY: &str = "session_id";
pub fn get_session_id(jar: &CookieJar) -> Option<i32> {
    let session_id_cookie = jar.get(SESSION_ID_COOKIE_KEY);

    if let Some(session_id) = session_id_cookie {
        let parsed_session_id = session_id.value().parse::<i32>();

        if let Ok(session_id) = parsed_session_id {
            return Some(session_id);
        }
    }

    None
}

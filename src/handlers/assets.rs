use axum::{
  http::header::{self, HeaderMap, HeaderValue},
  response::IntoResponse,
};
const CSS_INDEX_APP: &str = include_str!("../../static/tailwind-generated.css");

async fn asset(source: &'static [u8], ty: &'static str) -> impl IntoResponse {
  let mut headermap = HeaderMap::new();
  headermap.insert(header::CONTENT_TYPE, HeaderValue::from_static(ty));
  (headermap, source)
}

async fn css(source: &'static str) -> impl IntoResponse {
  asset(source.as_bytes(), &"text/css").await
}

pub async fn index_app_css() -> impl IntoResponse {
  css(CSS_INDEX_APP).await
}
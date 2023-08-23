use askama::Template;
use axum::{extract::State, response::{IntoResponse, Html}};

use crate::AppState;

#[derive(Clone)]
pub struct Todo {
    pub id: String,
    pub label: String,
    pub completed: bool,
}

#[derive(Template)]
#[template(path = "todos.html")]
struct TodosTemplate {
    todos: Vec<Todo>,
}

pub async fn todos(State(state): State<AppState>) -> impl IntoResponse {
    let todos = state.todos.lock().expect("Failed to lock the state");
    let template = TodosTemplate {
        todos: todos.to_vec(),
    };
    Html(template.to_string())
}

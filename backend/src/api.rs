//! HTTP handlers that implement the documented API contract.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};

use crate::model::{ErrorResponse, Todo, TodoCreate, TodoUpdate};
use crate::storage::Todos;

#[derive(Clone)]
pub struct AppState {
    pub todos: std::sync::Arc<tokio::sync::Mutex<Todos>>,
}

/// `GET /api/todos`
pub async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = state.todos.lock().await;
    Ok(Json(todos.list()))
}

/// `POST /api/todos`
pub async fn create_todo(
    State(state): State<AppState>,
    Json(input): Json<TodoCreate>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    if input.text.trim().is_empty() {
        return Err(AppError::bad_request("text must not be empty"));
    }
    let todo = state.todos.lock().await.create(input.text);
    Ok((StatusCode::CREATED, Json(todo)))
}

/// `PATCH /api/todos/:id`
pub async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<TodoUpdate>,
) -> Result<Json<Todo>, AppError> {
    let todo = state.todos.lock().await.update(&id, input);
    match todo {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::not_found(&format!("todo not found: {id}"))),
    }
}

/// `DELETE /api/todos/:id`
pub async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = state.todos.lock().await.delete(&id);
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(&format!("todo not found: {id}")))
    }
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/{id}", patch(update_todo).delete(delete_todo))
        .with_state(app_state)
        .without_v07_checks()
}

#[derive(Debug)]
enum AppError {
    NotFound { message: String },
    BadRequest { message: String },
}

impl AppError {
    fn not_found(message: &str) -> Self {
        Self::NotFound {
            message: message.to_string(),
        }
    }

    fn bad_request(message: &str) -> Self {
        Self::BadRequest {
            message: message.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let (status, message) = match self {
            AppError::NotFound { message } => (StatusCode::NOT_FOUND, message),
            AppError::BadRequest { message } => (StatusCode::BAD_REQUEST, message),
        };
        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

//! HTTP handlers that implement the documented API contract.

use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};

use crate::model::{ErrorResponse, Todo, TodoCreate, TodoUpdate};
use crate::storage::Todos;

#[derive(Clone)]
pub struct AppState {
    pub todos: Todos,
}

fn malformed_json() -> AppError {
    AppError::bad_request("malformed JSON")
}

/// `GET /api/todos`
pub async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = state.todos.list().await.map_err(|_| AppError::internal())?;
    Ok(Json(todos))
}

/// `POST /api/todos`
pub async fn create_todo(
    State(state): State<AppState>,
    body: Result<Json<TodoCreate>, JsonRejection>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let Json(input) = body.map_err(|_| malformed_json())?;
    if input.text.trim().is_empty() {
        return Err(AppError::bad_request("text must not be empty"));
    }
    let todo = state
        .todos
        .create(input.text)
        .await
        .map_err(|_| AppError::internal())?;
    Ok((StatusCode::CREATED, Json(todo)))
}

/// `PATCH /api/todos/:id`
pub async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Result<Json<TodoUpdate>, JsonRejection>,
) -> Result<Json<Todo>, AppError> {
    let Json(input) = body.map_err(|_| malformed_json())?;
    if input
        .text
        .as_ref()
        .is_some_and(|text| text.trim().is_empty())
    {
        return Err(AppError::bad_request("text must not be empty"));
    }
    let todo = state
        .todos
        .update(&id, input)
        .await
        .map_err(|_| AppError::internal())?;
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
    let deleted = state
        .todos
        .delete(&id)
        .await
        .map_err(|_| AppError::internal())?;
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
pub enum AppError {
    NotFound { message: String },
    BadRequest { message: String },
    Internal { message: String },
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

    fn internal() -> Self {
        Self::Internal {
            message: "database error".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let (status, message) = match self {
            AppError::NotFound { message } => (StatusCode::NOT_FOUND, message),
            AppError::BadRequest { message } => (StatusCode::BAD_REQUEST, message),
            AppError::Internal { message } => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

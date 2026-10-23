//! Data model and (de)serialization for the API contract.

use serde::{Deserialize, Serialize};

/// A single todo, as exposed on the wire.
///
/// Wire shape (camelCase):
/// ```json
/// {
///   "id": "todo-00000001",
///   "text": "Buy milk",
///   "completed": false,
///   "createdAt": "2026-10-23T12:00:00Z",
///   "updatedAt": "2026-10-23T12:00:00Z"
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Payload accepted by `POST /api/todos`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoCreate {
    pub text: String,
}

/// Partial payload accepted by `PATCH /api/todos/:id`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoUpdate {
    pub text: Option<String>,
    pub completed: Option<bool>,
}

/// Uniform error envelope for every non-2xx response.
///
/// ```json
/// { "error": "todo not found" }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

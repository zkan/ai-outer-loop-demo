//! HTTP tests for the todo CRUD contract.
//!
//! These drive the axum router in-process (no bound port) so `cargo test`
//! covers create, list, toggle, and delete, including the error envelope.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower::ServiceExt;

use crate::api::{router, AppState};
use crate::storage::Todos;

fn app() -> axum::Router {
    router(AppState {
        todos: Arc::new(Mutex::new(Todos::new())),
    })
}

async fn call(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let request_body = if let Some(value) = body {
        builder = builder.header("content-type", "application/json");
        Body::from(serde_json::to_vec(&value).expect("serialize request"))
    } else {
        Body::empty()
    };
    let request = builder.body(request_body).expect("build request");

    let response = app.oneshot(request).await.expect("router response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();

    let parsed = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, parsed)
}

fn error_message(body: &Value) -> &str {
    body.get("error")
        .and_then(Value::as_str)
        .expect("error envelope")
}

#[tokio::test]
async fn list_starts_empty() {
    let (status, body) = call(app(), "GET", "/api/todos", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn crud_create_list_toggle_delete() {
    let state = AppState {
        todos: Arc::new(Mutex::new(Todos::new())),
    };

    let (status, created) = call(
        router(state.clone()),
        "POST",
        "/api/todos",
        Some(json!({ "text": "Buy milk" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["text"], "Buy milk");
    assert_eq!(created["completed"], false);
    assert!(
        created.get("created_at").is_none(),
        "wire shape is camelCase"
    );
    assert!(created["createdAt"].as_str().unwrap().ends_with('Z'));
    assert_eq!(created["createdAt"], created["updatedAt"]);
    let id = created["id"].as_str().unwrap().to_string();
    assert!(id.starts_with("todo-"));

    let (status, list) = call(router(state.clone()), "GET", "/api/todos", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], id);

    let (status, toggled) = call(
        router(state.clone()),
        "PATCH",
        &format!("/api/todos/{id}"),
        Some(json!({ "completed": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(toggled["id"], id);
    assert_eq!(toggled["completed"], true);
    assert_eq!(toggled["text"], "Buy milk");

    let (status, list) = call(router(state.clone()), "GET", "/api/todos", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list[0]["completed"], true);

    let (status, body) = call(
        router(state.clone()),
        "DELETE",
        &format!("/api/todos/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(body, Value::Null);

    let (status, list) = call(router(state.clone()), "GET", "/api/todos", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list, json!([]));
}

#[tokio::test]
async fn create_rejects_blank_text() {
    let state = AppState {
        todos: Arc::new(Mutex::new(Todos::new())),
    };

    for text in ["", "   ", "\n\t"] {
        let (status, body) = call(
            router(state.clone()),
            "POST",
            "/api/todos",
            Some(json!({ "text": text })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "text={text:?}");
        assert!(!error_message(&body).is_empty());
        assert_eq!(body.as_object().unwrap().len(), 1);
    }

    let (status, list) = call(router(state), "GET", "/api/todos", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list, json!([]));
}

#[tokio::test]
async fn update_and_delete_unknown_id_return_error_envelope() {
    let (status, body) = call(
        app(),
        "PATCH",
        "/api/todos/todo-unknown",
        Some(json!({ "completed": true })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(error_message(&body).contains("todo-unknown"));

    let (status, body) = call(app(), "DELETE", "/api/todos/todo-unknown", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(error_message(&body).contains("todo-unknown"));
}

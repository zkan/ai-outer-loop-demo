//! HTTP tests for the todo CRUD contract.
//!
//! These drive the axum router in-process (no bound port) so `cargo test`
//! covers create, list, toggle, and delete, including the error envelope.

use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::api::{router, AppState};
use crate::storage::Todos;

fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = tempfile::tempdir().expect("temp dir");
    let n = N.fetch_add(1, Ordering::Relaxed);
    let path = dir.path().join(format!("todos-{n}.db"));
    (dir, path)
}

async fn test_state() -> (tempfile::TempDir, AppState) {
    let (dir, path) = temp_db();
    let todos = Todos::connect(&path).await.expect("connect");
    (dir, AppState { todos })
}

async fn app() -> (tempfile::TempDir, axum::Router) {
    let (dir, state) = test_state().await;
    (dir, router(state))
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
    let (_dir, app) = app().await;
    let (status, body) = call(app, "GET", "/api/todos", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[tokio::test]
async fn crud_create_list_toggle_delete() {
    let (_dir, state) = test_state().await;

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
    assert_rfc3339(created["createdAt"].as_str().unwrap());
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
    let (_dir, state) = test_state().await;

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
    let (_dir, api) = app().await;
    let (status, body) = call(
        api,
        "PATCH",
        "/api/todos/todo-unknown",
        Some(json!({ "completed": true })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(error_message(&body).contains("todo-unknown"));

    let (_dir, api) = app().await;
    let (status, body) = call(api, "DELETE", "/api/todos/todo-unknown", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(error_message(&body).contains("todo-unknown"));
}

#[tokio::test]
async fn patch_rejects_blank_text() {
    let (_dir, state) = test_state().await;
    let (status, created) = call(
        router(state.clone()),
        "POST",
        "/api/todos",
        Some(json!({ "text": "keep me" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap();

    for text in ["", "   ", "\n\t"] {
        let (status, body) = call(
            router(state.clone()),
            "PATCH",
            &format!("/api/todos/{id}"),
            Some(json!({ "text": text, "completed": true })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "text={text:?}");
        assert_eq!(error_message(&body), "text must not be empty");
        assert_eq!(body.as_object().unwrap().len(), 1);
    }

    let (status, list) = call(router(state), "GET", "/api/todos", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list[0]["text"], "keep me");
    assert_eq!(list[0]["completed"], false);
}

#[tokio::test]
async fn malformed_json_uses_error_envelope() {
    let cases = [
        ("POST", "/api/todos", "{"),
        ("POST", "/api/todos", "{\"text\":1}"),
        ("PATCH", "/api/todos/todo-00000001", "{\"completed\":"),
    ];
    for (method, uri, raw) in cases {
        let (_dir, app) = app().await;
        let (status, body) =
            call_raw(app, method, uri, Some("application/json"), raw.as_bytes()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{method} {raw}");
        assert_eq!(error_message(&body), "malformed JSON");
        assert_eq!(body.as_object().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn todos_persist_across_process_reopen() {
    let (dir, path) = temp_db();

    let id = {
        let state = AppState {
            todos: Todos::connect(&path).await.expect("connect"),
        };
        let (status, created) = call(
            router(state),
            "POST",
            "/api/todos",
            Some(json!({ "text": "persist me" })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_rfc3339(created["createdAt"].as_str().unwrap());
        created["id"].as_str().unwrap().to_string()
    };

    {
        let state = AppState {
            todos: Todos::connect(&path).await.expect("reopen"),
        };
        let (status, list) = call(router(state.clone()), "GET", "/api/todos", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(list[0]["id"], id);
        assert_eq!(list[0]["text"], "persist me");

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let (status, updated) = call(
            router(state),
            "PATCH",
            &format!("/api/todos/{id}"),
            Some(json!({ "completed": true })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(updated["completed"], true);
        assert_ne!(updated["createdAt"], updated["updatedAt"]);
        assert_rfc3339(updated["updatedAt"].as_str().unwrap());
    }

    {
        let state = AppState {
            todos: Todos::connect(&path).await.expect("reopen after update"),
        };
        let (status, list) = call(router(state.clone()), "GET", "/api/todos", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(list[0]["completed"], true);
        let (status, body) = call(router(state), "DELETE", &format!("/api/todos/{id}"), None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(body, Value::Null);
    }

    {
        let state = AppState {
            todos: Todos::connect(&path).await.expect("reopen after delete"),
        };
        let (status, list) = call(router(state), "GET", "/api/todos", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(list, json!([]));
    }

    drop(dir);
}

fn assert_rfc3339(ts: &str) {
    let parsed = DateTime::parse_from_rfc3339(ts).unwrap_or_else(|_| panic!("rfc3339: {ts}"));
    let skew = (Utc::now() - parsed.with_timezone(&Utc))
        .num_seconds()
        .abs();
    assert!(skew < 10, "{ts} is not the current UTC time");
}

async fn call_raw(
    app: axum::Router,
    method: &str,
    uri: &str,
    content_type: Option<&str>,
    body: &[u8],
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(content_type) = content_type {
        builder = builder.header("content-type", content_type);
    }
    let request = builder
        .body(Body::from(body.to_vec()))
        .expect("build request");
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

//! Binary entry point for the Outer Loop Todos API.

pub mod api;
mod model;
pub mod storage;

use std::net::SocketAddr;
use std::sync::Arc;

use api::AppState;

use axum::serve;
use tokio::sync::Mutex;

#[cfg(not(test))]
#[tokio::main]
async fn main() {
    let todos = Arc::new(Mutex::new(storage::Todos::new()));
    let app = api::router(AppState { todos });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("AI Outer Loop todos API listening on http://{addr}");
    serve(listener, app).await.unwrap();
}

//! Binary entry point for the Outer Loop Todos API.

pub mod api;
mod model;
pub mod storage;

use std::net::SocketAddr;

use api::AppState;

use axum::serve;

#[cfg(test)]
mod http_tests;

/// Bind address for the API. `API_HOST` defaults to 127.0.0.1 (or the name
/// `localhost`). `API_PORT` defaults to 3000.
#[cfg(not(test))]
fn listen_addr() -> SocketAddr {
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port_raw = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = match port_raw.parse() {
        Ok(port) if port != 0 => port,
        _ => {
            eprintln!("API_PORT must be an integer from 1 to 65535, got {port_raw:?}");
            std::process::exit(1);
        }
    };
    let ip = if host == "localhost" {
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    } else {
        match host.parse() {
            Ok(ip) => ip,
            Err(_) => {
                eprintln!("API_HOST must be an IP address or \"localhost\", got {host:?}");
                std::process::exit(1);
            }
        }
    };
    SocketAddr::new(ip, port)
}

#[cfg(not(test))]
#[tokio::main]
async fn main() {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "todos.db".to_string());
    let todos = match storage::Todos::connect(&db_path).await {
        Ok(todos) => todos,
        Err(err) => {
            eprintln!("failed to open database at {db_path}: {err}");
            std::process::exit(1);
        }
    };
    let app = api::router(AppState { todos });

    let addr = listen_addr();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("AI Outer Loop todos database at {db_path}");
    println!("AI Outer Loop todos API listening on http://{addr}");
    serve(listener, app).await.unwrap();
}

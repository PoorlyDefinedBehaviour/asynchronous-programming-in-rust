use std::time::Duration;

use axum::{extract::Path, response::IntoResponse, routing::get, Router};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/:delay/:request_id", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(Path((delay, request_id)): Path<(u32, String)>) -> impl IntoResponse {
    let duration = Duration::from_millis(delay as u64);
    println!("sleeping for {duration:?}");
    tokio::time::sleep(duration).await;
    format!("DONE: {request_id}")
}

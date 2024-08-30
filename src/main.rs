use askama_axum::Template;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Template)]
#[template(path = "index.html")] // Make sure this path is correct
pub struct IndexTemplate<'a> {
    pub title: &'a str,
    pub message: &'a str,
}

type SharedState = Arc<Mutex<Vec<bool>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(Mutex::new(vec![false; 1_000_000]));

    let app = Router::new()
        .route("/", get(index))
        .route("/toggle/:id", post(toggle))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(State(state): State<SharedState>) -> IndexTemplate<'static> {
    IndexTemplate {
        title: "One Million Checkboxes",
        message: "Toggle the checkboxes!",
    }
}

async fn toggle(Path(id): Path<usize>, State(state): State<SharedState>) -> &'static str {
    let mut checkboxes = state.lock().unwrap();
    if id < checkboxes.len() {
        checkboxes[id] = !checkboxes[id];
        "OK"
    } else {
        "Invalid ID"
    }
}

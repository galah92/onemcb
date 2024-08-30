use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: &'static str,
    pub message: &'static str,
    pub checkboxes: Vec<bool>,
}

#[derive(Template)]
#[template(path = "checkbox.html")]
pub struct CheckboxTemplate {
    pub index: usize,
    pub checked: bool,
}

type SharedState = Arc<Mutex<Vec<bool>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    const NUM_CHECKBOXES: usize = 1_000; // not exactly one million, but close enough
    let state = Arc::new(Mutex::new(vec![false; NUM_CHECKBOXES]));

    let app = Router::new()
        .route("/", get(index))
        .route("/toggle/:id", post(toggle))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[tracing::instrument(skip(state))]
async fn index(State(state): State<SharedState>) -> IndexTemplate {
    let checkboxes = state.lock().unwrap().clone();
    IndexTemplate {
        title: "One Million Checkboxes",
        message: "Toggle the checkboxes!",
        checkboxes,
    }
}

#[tracing::instrument(skip(state))]
async fn toggle(Path(index): Path<usize>, State(state): State<SharedState>) -> impl IntoResponse {
    let mut checkboxes = state.lock().unwrap();
    if index < checkboxes.len() {
        checkboxes[index] = !checkboxes[index];
        let checked = checkboxes[index];
        tracing::info!("Checkbox {} toggled to {}", index, checked);
        let template = CheckboxTemplate { index, checked };
        template.into_response()
    } else {
        tracing::warn!("Invalid checkbox ID: {}", index);
        "Invalid ID".into_response()
    }
}

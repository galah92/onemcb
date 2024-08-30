use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

type SharedState = Arc<Mutex<Vec<bool>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    const NUM_CHECKBOXES: usize = 1_000; // not exactly one million, but close enough
    let state = Arc::new(Mutex::new(vec![false; NUM_CHECKBOXES]));

    let app = Router::new()
        .route("/", get(index))
        .route("/toggle/:id", post(toggle))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into());
    let registry = tracing_subscriber::registry().with(filter);
    let format = std::env::var("RUST_LOG_FORMAT").unwrap_or("json".to_string());
    println!("format: {}", format);
    match format.as_str() {
        "pretty" => {
            let layer = tracing_subscriber::fmt::layer().pretty();
            registry.with(layer).init();
        }
        "gcp" => {
            let layer = tracing_stackdriver::layer().with_source_location(false);
            registry.with(layer).init();
        }
        _ => {
            let layer = tracing_subscriber::fmt::layer().json();
            registry.with(layer).init();
        }
    }
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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    pub title: &'static str,
    pub message: &'static str,
    pub checkboxes: Vec<bool>,
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

#[derive(Template)]
#[template(path = "checkbox.html")]
struct CheckboxTemplate {
    pub index: usize,
    pub checked: bool,
}

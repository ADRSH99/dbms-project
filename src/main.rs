use axum::{
    routing::{get, post},
    Json, Router,
    extract::State,
    response::Html,
};
use datafusion::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod adapters;

#[derive(Clone)]
struct AppState {
    ctx: Arc<SessionContext>,
}

#[derive(Deserialize)]
struct QueryRequest {
    sql: String,
}

#[derive(Serialize)]
struct QueryResponse {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    println!("❄️ IceWeave Federated Query Engine V2 Initializing...");

    // 1. Initialize DataFusion
    let ctx = Arc::new(SessionContext::new());

    // 2. Register all adapters (Fetches data into memory)
    adapters::register_all(&ctx).await?;

    // 3. Register all local CSVs dynamically
    println!("📊 Registering CSV Data from data/ directory...");
    if let Ok(entries) = std::fs::read_dir("data") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                let table_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                println!("  -> Registering CSV: {}", table_name);
                ctx.register_csv(&table_name, path.to_str().unwrap(), CsvReadOptions::new())
                    .await?;
            }
        }
    }

    println!("✅ All sources registered.");

    // 4. Set up the Web Server
    let state = AppState { ctx };

    let app = Router::new()
        .route("/", get(ui))
        .route("/query", post(run_query))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 IceWeave UI ready at http://localhost:3000");
    
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn ui() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

async fn run_query(
    State(state): State<AppState>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    println!("🔍 Executing SQL: {}", payload.sql);
    
    let df = match state.ctx.sql(&payload.sql).await {
        Ok(df) => df,
        Err(e) => return Json(QueryResponse {
            columns: vec![],
            rows: vec![],
            error: Some(e.to_string()),
        }),
    };

    let batches = match df.collect().await {
        Ok(b) => b,
        Err(e) => return Json(QueryResponse {
            columns: vec![],
            rows: vec![],
            error: Some(e.to_string()),
        }),
    };

    if batches.is_empty() {
        return Json(QueryResponse {
            columns: vec![],
            rows: vec![],
            error: None,
        });
    }

    let schema = batches[0].schema();
    let columns = schema.fields().iter().map(|f| f.name().clone()).collect();
    
    let mut rows = Vec::new();
    for batch in batches {
        for row_idx in 0..batch.num_rows() {
            let mut row = Vec::new();
            for col_idx in 0..batch.num_columns() {
                let col = batch.column(col_idx);
                
                let display_val = match datafusion::arrow::util::display::array_value_to_string(col, row_idx) {
                    Ok(s) => s,
                    Err(_) => "Error".to_string(),
                };
                row.push(display_val);
            }
            rows.push(row);
        }
    }

    Json(QueryResponse {
        columns,
        rows,
        error: None,
    })
}

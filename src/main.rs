use axum::{
    routing::{get, post, delete},
    Json, Router,
    extract::State,
    response::Html,
};
use datafusion::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use sqlx::postgres::PgPoolOptions;
use mongodb::{Client, options::ClientOptions};

mod adapters;

#[derive(Clone)]
pub struct AppState {
    pub ctx: Arc<RwLock<SessionContext>>,
    pub pg_pool: sqlx::PgPool,
    pub mongo_db: mongodb::Database,
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
    let ctx = SessionContext::new();

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

    // 4. Create persistent DB connections for reload
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/iceweave")
        .await
        .expect("Failed to create persistent PG pool");

    let mongo_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let mongo_client = Client::with_options(mongo_options)
        .expect("Failed to create MongoDB client");
    let mongo_db = mongo_client.database("iceweave");

    // 5. Set up the Web Server
    let state = AppState {
        ctx: Arc::new(RwLock::new(ctx)),
        pg_pool,
        mongo_db,
    };

    let app = Router::new()
        .route("/", get(ui))
        .route("/query", post(run_query))
        // REST API routes (in rest.rs)
        .route("/sources", get(adapters::rest::list_sources))
        .route("/upload/postgres", post(adapters::rest::upload_postgres))
        .route("/upload/mongo", post(adapters::rest::upload_mongo))
        .route("/upload/csv", post(adapters::rest::upload_csv))
        .route("/csv/{name}", delete(adapters::rest::delete_csv))
        .route("/reload", post(adapters::rest::reload_engine))
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
    
    let ctx = state.ctx.read().await;

    let df = match ctx.sql(&payload.sql).await {
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

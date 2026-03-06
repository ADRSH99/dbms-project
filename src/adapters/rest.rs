use axum::{
    extract::{Multipart, Path, State},
    Json,
};
use serde::Serialize;
use datafusion::prelude::*;

use crate::AppState;

#[derive(Serialize)]
pub struct SourceInfo {
    pub postgres_sql: Option<String>,
    pub mongo_js: Option<String>,
    pub csv_files: Vec<String>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub success: bool,
    pub message: String,
}

/// GET /sources — list current data source files
pub async fn list_sources() -> Json<SourceInfo> {
    let pg = if std::path::Path::new("init-scripts/postgres.sql").exists() {
        Some("postgres.sql".to_string())
    } else {
        None
    };
    let mongo = if std::path::Path::new("init-scripts/mongo.js").exists() {
        Some("mongo.js".to_string())
    } else {
        None
    };

    let mut csvs = Vec::new();
    if let Ok(entries) = std::fs::read_dir("data") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    csvs.push(name.to_string());
                }
            }
        }
    }
    csvs.sort();

    Json(SourceInfo {
        postgres_sql: pg,
        mongo_js: mongo,
        csv_files: csvs,
    })
}

/// POST /upload/postgres — upload a .sql file to init-scripts/
pub async fn upload_postgres(mut multipart: Multipart) -> Json<StatusResponse> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(data) = field.bytes().await.ok() {
            let content = String::from_utf8_lossy(&data);
            if let Err(e) = std::fs::write("init-scripts/postgres.sql", content.as_ref()) {
                return Json(StatusResponse {
                    success: false,
                    message: format!("Failed to save file: {}", e),
                });
            }
            return Json(StatusResponse {
                success: true,
                message: "PostgreSQL init script uploaded successfully.".to_string(),
            });
        }
    }
    Json(StatusResponse {
        success: false,
        message: "No file data received.".to_string(),
    })
}

/// POST /upload/mongo — upload a .js file to init-scripts/
pub async fn upload_mongo(mut multipart: Multipart) -> Json<StatusResponse> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(data) = field.bytes().await.ok() {
            let content = String::from_utf8_lossy(&data);
            if let Err(e) = std::fs::write("init-scripts/mongo.js", content.as_ref()) {
                return Json(StatusResponse {
                    success: false,
                    message: format!("Failed to save file: {}", e),
                });
            }
            return Json(StatusResponse {
                success: true,
                message: "MongoDB init script uploaded successfully.".to_string(),
            });
        }
    }
    Json(StatusResponse {
        success: false,
        message: "No file data received.".to_string(),
    })
}

/// POST /upload/csv — upload one or more .csv files to data/
pub async fn upload_csv(mut multipart: Multipart) -> Json<StatusResponse> {
    let mut count = 0;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("upload_{}.csv", count));

        // Ensure it ends with .csv
        let file_name = if file_name.ends_with(".csv") {
            file_name
        } else {
            format!("{}.csv", file_name)
        };

        if let Some(data) = field.bytes().await.ok() {
            let dest = format!("data/{}", file_name);
            if let Err(e) = std::fs::write(&dest, &data) {
                return Json(StatusResponse {
                    success: false,
                    message: format!("Failed to save {}: {}", file_name, e),
                });
            }
            count += 1;
        }
    }

    if count == 0 {
        return Json(StatusResponse {
            success: false,
            message: "No files received.".to_string(),
        });
    }

    Json(StatusResponse {
        success: true,
        message: format!("{} CSV file(s) uploaded successfully.", count),
    })
}

/// DELETE /csv/:name — delete a CSV file from data/
pub async fn delete_csv(Path(name): Path<String>) -> Json<StatusResponse> {
    let path = format!("data/{}", name);
    if !std::path::Path::new(&path).exists() {
        return Json(StatusResponse {
            success: false,
            message: format!("File '{}' not found.", name),
        });
    }
    if let Err(e) = std::fs::remove_file(&path) {
        return Json(StatusResponse {
            success: false,
            message: format!("Failed to delete: {}", e),
        });
    }
    Json(StatusResponse {
        success: true,
        message: format!("Deleted '{}'.", name),
    })
}

/// POST /reload — re-initialize databases from init scripts and reload all data
pub async fn reload_engine(State(state): State<AppState>) -> Json<StatusResponse> {
    println!("🔄 Reload requested — re-initializing all data sources...");

    // 1. Re-run PostgreSQL init script
    if std::path::Path::new("init-scripts/postgres.sql").exists() {
        let sql = match std::fs::read_to_string("init-scripts/postgres.sql") {
            Ok(s) => s,
            Err(e) => {
                return Json(StatusResponse {
                    success: false,
                    message: format!("Failed to read postgres.sql: {}", e),
                });
            }
        };

        println!("  🐘 Re-initializing PostgreSQL...");
        if let Err(e) = super::postgres::run_init_sql(&state.pg_pool, &sql).await {
            return Json(StatusResponse {
                success: false,
                message: format!("PostgreSQL init failed: {}", e),
            });
        }
    }

    // 2. Re-run MongoDB init script
    if std::path::Path::new("init-scripts/mongo.js").exists() {
        println!("  🍃 Re-initializing MongoDB...");
        if let Err(e) = super::mongodb::run_init_js().await {
            return Json(StatusResponse {
                success: false,
                message: format!("MongoDB init failed: {}", e),
            });
        }
    }

    // 3. Create a fresh DataFusion context and re-register everything
    println!("  📊 Rebuilding DataFusion context...");
    let new_ctx = SessionContext::new();

    // Register adapters
    if let Err(e) = super::register_all_into(&new_ctx, &state.pg_pool, &state.mongo_db).await {
        return Json(StatusResponse {
            success: false,
            message: format!("Failed to register adapters: {}", e),
        });
    }

    // Register CSVs
    if let Ok(entries) = std::fs::read_dir("data") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                let table_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                if let Err(e) = new_ctx
                    .register_csv(&table_name, path.to_str().unwrap(), CsvReadOptions::new())
                    .await
                {
                    eprintln!("  ⚠️ Failed to register CSV {}: {}", table_name, e);
                }
            }
        }
    }

    // Swap the context
    let mut ctx_guard = state.ctx.write().await;
    *ctx_guard = new_ctx;
    drop(ctx_guard);

    println!("✅ Engine reloaded successfully!");
    Json(StatusResponse {
        success: true,
        message: "Engine reloaded successfully! All data sources re-initialized.".to_string(),
    })
}

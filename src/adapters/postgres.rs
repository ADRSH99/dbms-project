use datafusion::arrow::array::{Float64Builder, Int32Builder, StringBuilder};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::error::Result;
use datafusion::prelude::*;
use sqlx::{postgres::PgPoolOptions, Row};
use std::sync::Arc;

/// Connect to Postgres and register tables (used on initial startup)
pub async fn register(ctx: &SessionContext) -> Result<()> {
    println!("🔌 Registering PostgreSQL Adapter (V2 In-Memory)...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/iceweave")
        .await
        .expect("Failed to connect to Postgres");

    register_with_pool(ctx, &pool).await
}

/// Register Postgres tables using an existing pool (used during reload)
pub async fn register_with_pool(ctx: &SessionContext, pool: &sqlx::PgPool) -> Result<()> {
    // Discover tables dynamically from the database
    let table_rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

    for table_row in table_rows {
        let table_name: String = table_row.get(0);

        // Discover columns for this table
        let col_rows = sqlx::query(
            "SELECT column_name, data_type FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1 ORDER BY ordinal_position"
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await
        .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

        let mut fields = Vec::new();
        let mut col_names = Vec::new();
        let mut col_types = Vec::new();

        for col_row in &col_rows {
            let col_name: String = col_row.get(0);
            let data_type: String = col_row.get(1);

            let arrow_type = pg_type_to_arrow(&data_type);
            fields.push(Field::new(&col_name, arrow_type.clone(), true));
            col_names.push(col_name);
            col_types.push(data_type);
        }

        let schema = Arc::new(Schema::new(fields));

        // Fetch all rows
        let query_str = format!("SELECT {} FROM {}", 
            col_names.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", "),
            table_name
        );
        let rows = sqlx::query(&query_str)
            .fetch_all(pool)
            .await
            .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

        // Build arrays dynamically based on column types
        let mut builders: Vec<ColumnBuilder> = col_types.iter().map(|t| match pg_type_to_arrow(t) {
            DataType::Int32 => ColumnBuilder::Int32(Int32Builder::new()),
            DataType::Float64 => ColumnBuilder::Float64(Float64Builder::new()),
            _ => ColumnBuilder::Utf8(StringBuilder::new()),
        }).collect();

        for row in &rows {
            for (i, col_type) in col_types.iter().enumerate() {
                match &mut builders[i] {
                    ColumnBuilder::Int32(b) => {
                        let val: Option<i32> = row.try_get(i).ok();
                        match val {
                            Some(v) => b.append_value(v),
                            None => b.append_null(),
                        }
                    }
                    ColumnBuilder::Float64(b) => {
                        // Handle both FLOAT8 and NUMERIC types
                        if col_type == "numeric" {
                            let val: Option<rust_decimal::Decimal> = row.try_get(i).ok();
                            match val {
                                Some(v) => b.append_value(v.to_string().parse().unwrap_or(0.0)),
                                None => b.append_null(),
                            }
                        } else {
                            let val: Option<f64> = row.try_get(i).ok();
                            match val {
                                Some(v) => b.append_value(v),
                                None => b.append_null(),
                            }
                        }
                    }
                    ColumnBuilder::Utf8(b) => {
                        let val: Option<String> = row.try_get(i).ok();
                        match val {
                            Some(v) => b.append_value(v),
                            None => b.append_null(),
                        }
                    }
                }
            }
        }

        let arrays: Vec<Arc<dyn datafusion::arrow::array::Array>> = builders.into_iter().map(|b| match b {
            ColumnBuilder::Int32(mut b) => Arc::new(b.finish()) as Arc<dyn datafusion::arrow::array::Array>,
            ColumnBuilder::Float64(mut b) => Arc::new(b.finish()) as Arc<dyn datafusion::arrow::array::Array>,
            ColumnBuilder::Utf8(mut b) => Arc::new(b.finish()) as Arc<dyn datafusion::arrow::array::Array>,
        }).collect();

        let batch = RecordBatch::try_new(schema.clone(), arrays)
            .map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?;

        let mem_table = MemTable::try_new(schema, vec![vec![batch]])?;
        ctx.register_table(&table_name, Arc::new(mem_table))?;
        println!("  -> Registered PG Table: {}", table_name);
    }

    println!("✅ All PostgreSQL tables loaded into memory.");
    Ok(())
}

/// Run a SQL init script against Postgres (drop + recreate)
pub async fn run_init_sql(pool: &sqlx::PgPool, sql: &str) -> std::result::Result<(), String> {
    // First, drop all existing tables in public schema
    let drop_rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to list tables: {}", e))?;

    for row in drop_rows {
        let table_name: String = row.get(0);
        let drop_sql = format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table_name);
        sqlx::query(&drop_sql)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to drop table {}: {}", table_name, e))?;
    }

    // Execute the init SQL by splitting on semicolon
    for stmt in sql.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() { continue; }
        sqlx::query(stmt)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to execute init SQL statement '{}': {}", stmt, e))?;
    }

    println!("  ✅ PostgreSQL re-initialized.");
    Ok(())
}

enum ColumnBuilder {
    Int32(Int32Builder),
    Float64(Float64Builder),
    Utf8(StringBuilder),
}

fn pg_type_to_arrow(pg_type: &str) -> DataType {
    match pg_type {
        "integer" | "int4" | "smallint" | "int2" => DataType::Int32,
        "bigint" | "int8" => DataType::Int64,
        "real" | "float4" => DataType::Float32,
        "double precision" | "float8" | "numeric" => DataType::Float64,
        _ => DataType::Utf8,
    }
}

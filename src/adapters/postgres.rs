use datafusion::arrow::array::{Float64Builder, Int32Builder, StringBuilder};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::error::Result;
use datafusion::prelude::*;
use sqlx::{postgres::PgPoolOptions, Row};
use std::sync::Arc;

pub async fn register(ctx: &SessionContext) -> Result<()> {
    println!("🔌 Registering PostgreSQL Adapter (V2 In-Memory)...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/iceweave")
        .await
        .expect("Failed to connect to Postgres");

    // Table 1: orders
    register_table(ctx, &pool, "orders", "SELECT id, user_id, amount, product FROM orders", Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("user_id", DataType::Int32, false),
        Field::new("amount", DataType::Float64, false),
        Field::new("product", DataType::Utf8, false),
    ]))).await?;

    // Table 2: customers
    register_table(ctx, &pool, "customers", "SELECT id, name, email FROM customers", Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("email", DataType::Utf8, false),
    ]))).await?;

    // Table 3: reviews
    register_table(ctx, &pool, "reviews", "SELECT id, order_id, rating, comment FROM reviews", Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("order_id", DataType::Int32, false),
        Field::new("rating", DataType::Int32, false),
        Field::new("comment", DataType::Utf8, false),
    ]))).await?;

    println!("✅ All PostgreSQL tables loaded into memory.");
    Ok(())
}

async fn register_table(
    ctx: &SessionContext, 
    pool: &sqlx::PgPool, 
    table_name: &str, 
    query: &str, 
    schema: Arc<Schema>
) -> Result<()> {
    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

    let mut id_builder = Int32Builder::new();
    let mut col1_builder = Int32Builder::new(); // Used for user_id or product_id
    let mut float_builder = Float64Builder::new(); // Used for amount
    let mut string_builder = StringBuilder::new(); // Used for product, name, email, comment
    let mut rating_builder = Int32Builder::new(); // Used for rating

    let mut batches_vec = Vec::new();

    // Note: Since schemas vary, we'll use a more generic approach or match per table
    // For this project V2, we'll keep it specific to our 3 tables for clarity
    match table_name {
        "orders" => {
            let mut u_id = Int32Builder::new();
            let mut amt = Float64Builder::new();
            let mut prod = StringBuilder::new();
            let mut ids = Int32Builder::new();
            for row in rows {
                ids.append_value(row.get(0));
                u_id.append_value(row.get(1));
                let amount: rust_decimal::Decimal = row.get(2);
                amt.append_value(amount.to_string().parse().unwrap_or(0.0));
                prod.append_value(row.get::<String, _>(3));
            }
            let batch = RecordBatch::try_new(schema.clone(), vec![
                Arc::new(ids.finish()), Arc::new(u_id.finish()), Arc::new(amt.finish()), Arc::new(prod.finish())
            ]).map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?;
            batches_vec.push(batch);
        },
        "customers" => {
            let mut ids = Int32Builder::new();
            let mut names = StringBuilder::new();
            let mut emails = StringBuilder::new();
            for row in rows {
                ids.append_value(row.get(0));
                names.append_value(row.get::<String, _>(1));
                emails.append_value(row.get::<String, _>(2));
            }
            let batch = RecordBatch::try_new(schema.clone(), vec![
                Arc::new(ids.finish()), Arc::new(names.finish()), Arc::new(emails.finish())
            ]).map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?;
            batches_vec.push(batch);
        },
        "reviews" => {
            let mut ids = Int32Builder::new();
            let mut p_ids = Int32Builder::new();
            let mut rats = Int32Builder::new();
            let mut comms = StringBuilder::new();
            for row in rows {
                ids.append_value(row.get(0));
                p_ids.append_value(row.get(1));
                rats.append_value(row.get(2));
                comms.append_value(row.get::<String, _>(3));
            }
            let batch = RecordBatch::try_new(schema.clone(), vec![
                Arc::new(ids.finish()), Arc::new(p_ids.finish()), Arc::new(rats.finish()), Arc::new(comms.finish())
            ]).map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?;
            batches_vec.push(batch);
        },
        _ => {}
    }

    let mem_table = MemTable::try_new(schema, vec![batches_vec])?;
    ctx.register_table(table_name, Arc::new(mem_table))?;
    println!("  -> Registered PG Table: {}", table_name);
    Ok(())
}

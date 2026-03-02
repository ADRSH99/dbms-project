use datafusion::arrow::array::{Int32Builder, StringBuilder};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::error::Result;
use datafusion::prelude::*;
use futures::stream::StreamExt;
use mongodb::{Client, options::ClientOptions, bson::doc};
use std::sync::Arc;

pub async fn register(ctx: &SessionContext) -> Result<()> {
    println!("🔌 Registering MongoDB Adapter (V2 In-Memory)...");

    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");
    let db = client.database("iceweave");

    // Collection 1: inventory
    register_collection(ctx, &db, "inventory", Arc::new(Schema::new(vec![
        Field::new("item", DataType::Utf8, false),
        Field::new("qty", DataType::Int32, false),
    ]))).await?;

    // Collection 2: stock_logs
    register_collection(ctx, &db, "stock_logs", Arc::new(Schema::new(vec![
        Field::new("item", DataType::Utf8, false),
        Field::new("action", DataType::Utf8, false),
        Field::new("amount", DataType::Int32, false),
        Field::new("date", DataType::Utf8, false),
    ]))).await?;

    println!("✅ All MongoDB collections loaded into memory.");
    Ok(())
}

async fn register_collection(
    ctx: &SessionContext,
    db: &mongodb::Database,
    coll_name: &str,
    schema: Arc<Schema>
) -> Result<()> {
    let collection = db.collection::<mongodb::bson::Document>(coll_name);
    let mut cursor = collection.find(mongodb::bson::doc! {}).await
        .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

    let mut batches_vec = Vec::new();
    
    match coll_name {
        "inventory" => {
            let mut item_b = StringBuilder::new();
            let mut qty_b = Int32Builder::new();
            while let Some(result) = cursor.next().await {
                let doc = result.map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;
                item_b.append_value(doc.get_str("item").unwrap_or(""));
                qty_b.append_value(doc.get_i32("qty").unwrap_or(0));
            }
            let batch = RecordBatch::try_new(schema.clone(), vec![
                Arc::new(item_b.finish()), Arc::new(qty_b.finish())
            ]).map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?;
            batches_vec.push(batch);
        },
        "stock_logs" => {
            let mut item_b = StringBuilder::new();
            let mut act_b = StringBuilder::new();
            let mut amt_b = Int32Builder::new();
            let mut date_b = StringBuilder::new();
            while let Some(result) = cursor.next().await {
                let doc = result.map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;
                item_b.append_value(doc.get_str("item").unwrap_or(""));
                act_b.append_value(doc.get_str("action").unwrap_or(""));
                amt_b.append_value(doc.get_i32("amount").unwrap_or(0));
                date_b.append_value(doc.get_str("date").unwrap_or(""));
            }
            let batch = RecordBatch::try_new(schema.clone(), vec![
                Arc::new(item_b.finish()), Arc::new(act_b.finish()), Arc::new(amt_b.finish()), Arc::new(date_b.finish())
            ]).map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?;
            batches_vec.push(batch);
        },
        _ => {}
    }

    let mem_table = MemTable::try_new(schema, vec![batches_vec])?;
    ctx.register_table(coll_name, Arc::new(mem_table))?;
    println!("  -> Registered Mongo Coll: {}", coll_name);
    Ok(())
}

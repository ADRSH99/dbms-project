use datafusion::arrow::array::{Int32Builder, StringBuilder, Float64Builder};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::error::Result;
use datafusion::prelude::*;
use futures::stream::StreamExt;
use mongodb::{Client, options::ClientOptions};
use std::sync::Arc;

/// Connect to MongoDB and register collections (used on initial startup)
pub async fn register(ctx: &SessionContext) -> Result<()> {
    println!("🔌 Registering MongoDB Adapter (V2 In-Memory)...");

    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");
    let db = client.database("iceweave");

    register_with_db(ctx, &db).await
}

/// Register MongoDB collections using an existing db handle (used during reload)
pub async fn register_with_db(ctx: &SessionContext, db: &mongodb::Database) -> Result<()> {
    // Discover collections dynamically
    let collection_names = db.list_collection_names().await
        .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

    for coll_name in collection_names {
        // Skip system collections
        if coll_name.starts_with("system.") {
            continue;
        }

        let collection = db.collection::<mongodb::bson::Document>(&coll_name);

        // Peek at the first document to discover schema
        let mut peek_cursor = collection.find(mongodb::bson::doc! {}).await
            .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

        let sample_doc = match peek_cursor.next().await {
            Some(Ok(doc)) => doc,
            _ => {
                println!("  -> Skipping empty collection: {}", coll_name);
                continue;
            }
        };

        // Build schema from the sample document
        let mut fields = Vec::new();
        let mut field_names = Vec::new();
        let mut field_types = Vec::new();

        for (key, value) in sample_doc.iter() {
            if key == "_id" {
                continue; // Skip MongoDB's internal _id
            }
            let (arrow_type, bson_type) = bson_to_arrow_type(value);
            fields.push(Field::new(key, arrow_type, true));
            field_names.push(key.clone());
            field_types.push(bson_type);
        }

        let schema = Arc::new(Schema::new(fields));

        // Now fetch all documents
        let mut cursor = collection.find(mongodb::bson::doc! {}).await
            .map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;

        // Build column builders based on discovered types
        let mut builders: Vec<MongoColumnBuilder> = field_types.iter().map(|t| match t.as_str() {
            "int32" => MongoColumnBuilder::Int32(Int32Builder::new()),
            "float64" => MongoColumnBuilder::Float64(Float64Builder::new()),
            _ => MongoColumnBuilder::Utf8(StringBuilder::new()),
        }).collect();

        while let Some(result) = cursor.next().await {
            let doc = result.map_err(|e| datafusion::error::DataFusionError::External(Box::new(e)))?;
            for (i, field_name) in field_names.iter().enumerate() {
                match &mut builders[i] {
                    MongoColumnBuilder::Int32(b) => {
                        b.append_value(doc.get_i32(field_name).unwrap_or(0));
                    }
                    MongoColumnBuilder::Float64(b) => {
                        let val = if let Ok(v) = doc.get_f64(field_name) {
                            v
                        } else if let Ok(v) = doc.get_i32(field_name) {
                            v as f64
                        } else {
                            0.0
                        };
                        b.append_value(val);
                    }
                    MongoColumnBuilder::Utf8(b) => {
                        b.append_value(doc.get_str(field_name).unwrap_or(""));
                    }
                }
            }
        }

        let arrays: Vec<Arc<dyn datafusion::arrow::array::Array>> = builders.into_iter().map(|b| match b {
            MongoColumnBuilder::Int32(mut b) => Arc::new(b.finish()) as Arc<dyn datafusion::arrow::array::Array>,
            MongoColumnBuilder::Float64(mut b) => Arc::new(b.finish()) as Arc<dyn datafusion::arrow::array::Array>,
            MongoColumnBuilder::Utf8(mut b) => Arc::new(b.finish()) as Arc<dyn datafusion::arrow::array::Array>,
        }).collect();

        if arrays.is_empty() {
            continue;
        }

        let batch = RecordBatch::try_new(schema.clone(), arrays)
            .map_err(|e| datafusion::error::DataFusionError::ArrowError(Box::new(e), None))?;

        let mem_table = MemTable::try_new(schema, vec![vec![batch]])?;
        ctx.register_table(&coll_name, Arc::new(mem_table))?;
        println!("  -> Registered Mongo Coll: {}", coll_name);
    }

    println!("✅ All MongoDB collections loaded into memory.");
    Ok(())
}

/// Re-initialize MongoDB by dropping the database and running the init script via mongosh
pub async fn run_init_js() -> std::result::Result<(), String> {
    // Drop the iceweave database first
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .map_err(|e| format!("Failed to connect to MongoDB: {}", e))?;
    let client = Client::with_options(client_options)
        .map_err(|e| format!("Failed to create MongoDB client: {}", e))?;
    
    client.database("iceweave").drop().await
        .map_err(|e| format!("Failed to drop database: {}", e))?;

    // Run the init script using mongosh
    let output = tokio::process::Command::new("mongosh")
        .arg("--quiet")
        .arg("mongodb://localhost:27017/iceweave")
        .arg("--file")
        .arg("init-scripts/mongo.js")
        .output()
        .await
        .map_err(|e| format!("Failed to run mongosh: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mongosh failed: {}", stderr));
    }

    println!("  ✅ MongoDB re-initialized.");
    Ok(())
}

enum MongoColumnBuilder {
    Int32(Int32Builder),
    Float64(Float64Builder),
    Utf8(StringBuilder),
}

fn bson_to_arrow_type(value: &mongodb::bson::Bson) -> (DataType, String) {
    match value {
        mongodb::bson::Bson::Int32(_) => (DataType::Int32, "int32".to_string()),
        mongodb::bson::Bson::Int64(_) => (DataType::Int64, "int32".to_string()), // fit into int32 for simplicity
        mongodb::bson::Bson::Double(_) => (DataType::Float64, "float64".to_string()),
        mongodb::bson::Bson::String(_) => (DataType::Utf8, "string".to_string()),
        _ => (DataType::Utf8, "string".to_string()),
    }
}

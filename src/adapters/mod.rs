pub mod postgres;
pub mod mongodb;
pub mod s3;
pub mod rest;

use datafusion::prelude::SessionContext;

/// Registers all federated data adapters with the given DataFusion context.
pub async fn register_all(ctx: &SessionContext) -> datafusion::error::Result<()> {
    println!("🔌 Registering DataFusion Adapters...");
    
    // Register the S3/File system adapter
    s3::register(ctx).await?;

    // Register the PostgreSQL adapter
    postgres::register(ctx).await?;

    // Register the MongoDB adapter
    mongodb::register(ctx).await?;

    Ok(())
}

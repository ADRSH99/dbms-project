pub mod postgres;
pub mod mongodb;
pub mod s3;
pub mod rest;

use datafusion::prelude::SessionContext;

/// Registers all federated data adapters with the given DataFusion context.
/// Used on initial startup.
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

/// Re-registers all data adapters using existing DB connections.
/// Used during engine reload.
pub async fn register_all_into(
    ctx: &SessionContext,
    pg_pool: &sqlx::PgPool,
    mongo_db: &::mongodb::Database,
) -> datafusion::error::Result<()> {
    // Register the S3/File system adapter
    s3::register(ctx).await?;

    // Register PostgreSQL tables using existing pool
    postgres::register_with_pool(ctx, pg_pool).await?;

    // Register MongoDB collections using existing db handle
    mongodb::register_with_db(ctx, mongo_db).await?;

    Ok(())
}

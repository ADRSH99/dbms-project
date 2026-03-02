use datafusion::prelude::*;
use datafusion::error::Result;
use object_store::local::LocalFileSystem;
use url::Url;
use std::sync::Arc;

pub async fn register(ctx: &SessionContext) -> Result<()> {
    println!("🔌 Registering S3/Local Storage Adapter...");

    // For demonstration, we'll register the local file system to simulate file-based queries
    // S3 requires configuring object_store::aws::AmazonS3Builder which needs credentials
    let local_store = Arc::new(LocalFileSystem::new());
    ctx.runtime_env()
        .register_object_store(&Url::parse("file://").unwrap(), local_store);

    Ok(())
}

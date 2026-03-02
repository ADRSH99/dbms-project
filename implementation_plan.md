# IceWeave Implementation Plan

The `dbms-project` directory currently only contains your project report ([IceWeave_Project_Report_Updated.docx](file:///home/adarsh/Documents/dbmsproj/dbms-project/IceWeave_Project_Report_Updated.docx)) and a [README.md](file:///home/adarsh/Documents/dbmsproj/dbms-project/README.md). No code has been implemented yet. Based on the report, we will build a federated query engine from scratch using Rust and Apache DataFusion.

## Goal Description

Build **IceWeave**, a federated query engine that enables cross-source SQL queries. We will establish the core execution engine and add modular custom connectors (adapters) for PostgreSQL, MongoDB, S3-compatible storage, and REST APIs, avoiding any centralized data warehousing or redundant ETL pipelines.

## Proposed Architecture & File Structure

We will initialize a new Rust application and establish the following modular directory structure:

### Base Project Configuration
#### [NEW] `Cargo.toml`
Defines project metadata and core dependencies including `tokio` (for async execution), `datafusion` (query engine), `object_store` (for S3), and `sqlx`/`mongodb` for database connectivity.

### Application Logic Core
#### [NEW] `src/main.rs`
The entry point that initializes the query engine, registers the external table providers, and handles the CLI or API for incoming user queries.

#### [NEW] `src/engine/mod.rs`
Wraps the `SessionContext` from Apache DataFusion. This handles parsing, query planning, and physical execution scheduling.

#### [NEW] `src/adapters/mod.rs`
The Federated Adapter Layer that defines the trait/interface for integrating different data sources into DataFusion's `TableProvider` trait.

#### [NEW] `src/adapters/postgres.rs`
Implementation of the PostgreSQL connector mapping PG relations to DataFusion schemas.

#### [NEW] `src/adapters/mongodb.rs`
Implementation of the MongoDB connector for querying document collections.

#### [NEW] `src/adapters/s3.rs`
Object storage integration utilizing DataFusion's native `object_store` integration for Parquet/CSV scanning.

#### [NEW] `src/adapters/rest.rs`
REST API connector for fetching JSON payloads and mapping them to Arrow record batches.

---

## Rollout Strategy (How to Start Building)

We will build the system iteratively:
1. **Phase 1 (Core Engine):** Initialize the Rust project, add dependencies, and set up a basic DataFusion `SessionContext` capable of running a trivial in-memory SQL query.
2. **Phase 2 (File Storage Adapter):** Implement the S3/Local CSV/Parquet reader first, as it is the most natively supported format in DataFusion.
3. **Phase 3 (Relational Adapter):** Implement the PostgreSQL connector using `sqlx` and implement the `TableProvider` trait.
4. **Phase 4 (Document & API Adapters):** Build the MongoDB and REST API connectors.
5. **Phase 5 (Cross-Source Execution):** Test complex JOIN queries that span across these distributed TableProviders, ensuring predicate pushdown is optimally utilized.

---

## Verification Plan

### Automated Tests
- Unit tests will be added alongside every new `Adapter` to verify connection and schema inference.
- Command to run: `cargo test`

### Manual Verification
- We will spin up local Docker containers for PostgreSQL, MongoDB, and MinIO (S3).
- We will seed these databases with mock relational, document, and blob data.
- We will run `cargo run` and pass federated SQL queries joining data from all 3 sources to ensure the unified query engine functions properly.

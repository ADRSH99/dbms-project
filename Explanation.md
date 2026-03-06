# IceWeave: Deep-Dive Technical Explanation (V4.0)

This document provides a comprehensive, extremely detailed technical breakdown of every major file and code snippet residing within the IceWeave Federated Query Engine V4.

---

## 🏗️ 1. Project Management & Core Foundation

### `Cargo.toml`
**Role**: The heart of the project's dependency management and compilation strategy.
**Detailed Explanation**: 
This file instructs the Rust compiler (`cargo`) on what external libraries (crates) to pull. 
- It imports **Apache DataFusion** (`datafusion = "52.2.0"`) which provides the SQL parser, logical query planner, and physical execution framework. 
- It imports **Apache Arrow** (`arrow = "58.0.0"`) for the mathematically optimal columnar memory layout. 
- **Axum** (`axum = { version = "0.8.8", features = ["multipart"] }`) is used for the extremely fast web server, with the `multipart` feature explicitly enabled in V4 to support binary file uploads from the browser. 
- **SQLx** and **MongoDB** drivers handle asynchronous database connectivity.

### `docker-compose.yml`
**Role**: The absolute infrastructure definition for security and database deployment.
**Detailed Explanation**: 
This YAML configuration file completely automates the spin-up of enterprise-grade database instances.
- **PostgreSQL Service (`iceweave-postgres`)**: Pulls the official `postgres:15` image. It isolates the environment variables (user/password) and exclusively mounts `./init-scripts/postgres.sql` to `/docker-entrypoint-initdb.d/init.sql`. This ensures the database boots up with our exact schema perfectly deterministically.
- **MongoDB Service (`iceweave-mongo`)**: Pulls `mongo:6` and crucially mounts `./init-scripts/mongo.js`. 
- **Security Isolation**: Both containers run in a localized Docker Bridge network. They are sandboxed from the host operating system, guaranteeing that rogue software cannot corrupt the data. Only the IceWeave engine communicates over the securely mapped ports (5432 and 27017).

---

## ⚡ 2. Engine Orchestration & Web Gateway

### `src/main.rs`
**Role**: The central application loop, Request Router, and Execution Engine Coordinator.
**Detailed Breakdown**:
- **`AppState` struct**: 
  ```rust
  pub struct AppState {
      pub ctx: Arc<RwLock<SessionContext>>,
      pub pg_pool: sqlx::PgPool,
      pub mongo_db: mongodb::Database,
  }
  ```
  *Explanation*: This is the heartbeat of V4. The DataFusion `SessionContext` is wrapped in an `Arc<RwLock>`. `Arc` guarantees safe multi-threading ownership, while `RwLock` allows hundreds of simultaneous read queries but permits the `/reload` endpoint to acquire an exclusive write lock to completely swap the engine context on the fly without crashing. Furthermore, `pg_pool` and `mongo_db` hold persistent TCP connection pools so they don't have to be recreated during a reload.

- **Dynamic CSV Scanner**: 
  *Explanation*: Upon boot, `std::fs::read_dir("data")` traverses the local `./data` folder. Every single `.csv` file found is instantaneously registered into the DataFusion context utilizing `ctx.register_csv()`.

- **Web Server Initialization (`axum::serve`)**: 
  *Explanation*: Utilizes Tokio's asynchronous TCP listener binding to `0.0.0.0:3000`. It registers routes mapping HTTP requests to functions: `/` to the User Interface, `/query` to the engine executor, and all `/upload/*` routes to the REST adapter.

- **`run_query()` Function**:
  *Explanation*: This function intercepts inbound JSON containing SQL. It acquires a read-lock on the context (`state.ctx.read().await`), feeds the string to DataFusion (`ctx.sql(&payload.sql).await`), and systematically executes the mapped physical plan. Finally, it iterates over the complex binary `RecordBatches`, converting Arrow datatypes securely into simple string arrays `Vec<Vec<String>>` for the browser to consume.

---

## 🔌 3. Data Adapters (The Translators)

### `src/adapters/mod.rs`
**Role**: The central adapter registration hub.
**Detailed Breakdown**:
- Contains `register_all()` which wraps the initialization of all systems.
- In V4, it introduces `register_all_into(ctx, pg_pool, mongo_db)`, a crucial function that takes the persistent database connections from `AppState` and dynamically re-wires the newly built `SessionContext` during a Hot-Reload event, ensuring zero connection downtime.

### `src/adapters/postgres.rs` (Relational SQL Adapter)
**Role**: Transforms Postgres relational disk-rows into high-speed Apache Arrow Memory Batches.
**Detailed Breakdown**:
- **Dynamic Schema Discovery**: 
  ```rust
  sqlx::query("SELECT column_name, data_type FROM information_schema.columns ...")
  ```
  *Explanation*: Instead of hardcoding tables like in V2/V3, V4 actively queries the `information_schema`. It figures out exactly what tables exist and what their columns are made of.
- **Data Extractor**: It matches Postgres types (like `FLOAT8` or `INT4`) to Arrow builders (`Float64Builder`, `Int32Builder`). It executes a dynamic `SELECT *` string, manually iterating over thousands of rows and pushing them into vectorized RAM arrays.
- **`run_init_sql()`**: 
  *Explanation*: This V4 feature takes an uploaded `.sql` string, iteratively drops existing tables using `DROP TABLE IF EXISTS CASCADE`, and then loops through the new script via `.split(';')`, executing each statement sequentially to completely rebuild the database state dynamically.

### `src/adapters/mongodb.rs` (NoSQL Document Adapter)
**Role**: Deflattens unstructured BSON into rigid Columnar formats.
**Detailed Breakdown**:
- **Automatic Type Inference**: 
  *Explanation*: It peeks at the very first document in a collection using `peek_cursor.next().await`. It analyzes keys (skipping MongoDB's internal `_id`) and maps `mongodb::bson::Bson::Double` to Arrow `Float64`, dynamically creating a rigid schema for an inherently schema-less database.
- **`run_init_js()`**: 
  *Explanation*: The V4 update handles uploaded `.js` scripts. It forcefully drops the entire `iceweave` database via the Rust driver (`client.database("iceweave").drop()`), then spawns a child operating system process utilizing `tokio::process::Command` to trigger the `mongosh` terminal utility. This executes the sophisticated Javascript logic efficiently.

### `src/adapters/rest.rs` (V4 REST API Manager)
**Role**: The highly-concurrent file management and engine orchestration interface.
**Detailed Breakdown**:
- **`upload_postgres()`, `upload_mongo()`, `upload_csv()`**: 
  *Explanation*: These endpoints securely accept binary `Multipart` streams natively from the browser. They decode the byte streams safely using `String::from_utf8_lossy()` and commit the exact bytes to the physical Host File System (`init-scripts/` or `data/`).
- **`reload_engine()`**: 
  *Explanation*: The crown jewel of V4. 
  1. It triggers `pg.run_init_sql()` and `mongo.run_init_js()` to permanently altering the local Docker databases based on the new files.
  2. It spins up a brand new, empty DataFusion `SessionContext`.
  3. It executes `register_all_into` to aggressively pull all the newly restructured data entirely back into RAM.
  4. Finally, it seizes the `RwLock` write-guard over `main.rs`'s AppState and atomically swaps the old engine context with the new one. Users instantly query the new data architecture.

---

## 🎨 4. User Interface Layer

### `index.html`
**Role**: The fully interactive User Experience portal.
**Detailed Breakdown**:
- **Architecture**: It's a monolithic single-page application heavily relying on `fetch` asynchronous API calls, styled with deep "Glassmorphism" variables (`var(--surface)`).
- **Data Source Manager Panel (V4)**: 
  *Explanation*: Uses HTML5 `<input type="file" accept=".sql">` APIs mapped to invisible drop zones. When a user clicks, picks a file, `uploadFile()` converts it into a pure `FormData` object and POSTs it directly to the `/upload` endpoints.
- **Reload Functionality**: The `reloadEngine()` JS function halts UI interactivity, calls the `/reload` backend, and dynamically cascades the success state without requiring a browser refresh.
- **Categorized Query Chips**: Contains complex templates meticulously categorizing "Single Source", "Cross-Source", and "Full Federation" queries, heavily demonstrating the mathematical beauty of joining PostgreSQL, MongoDB, and local CSVs in sub-millisecond real-time logic.

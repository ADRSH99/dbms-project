# IceWeave: Deep-Dive Technical Explanation (V2.1)

This document provides a comprehensive, file-by-file technical breakdown of the IceWeave Federated Query Engine.

---

## 🏗️ 1. Project Management & Core
### `Cargo.toml`
The heart of our dependency management. It imports **DataFusion** for the SQL brain, **Arrow** for memory management, **Axum** for the web server, and **SQLx/MongoDB** for database connectivity. It ensures all versions are compatible and that features like `tokio` multi-threading are enabled.

---

## 🔌 2. Data Adapters (The Connectors)

### `src/adapters/mod.rs`
**Purpose**: The central registration hub.
- It exposes a single function `register_all()` which acts as a master switch. When called, it sequentially triggers the registration of the S3, PostgreSQL, MongoDB, and REST adapters.

### `src/adapters/postgres.rs` (PostgreSQL / SQL)
**Purpose**: Transforms relational database rows into high-speed memory batches.
- **`register()`**: Establishes a connection to the Dockerized Postgres instance.
- **`register_table()`**: A generic function that takes a SQL query (e.g., `SELECT * FROM orders`), fetches the results using `sqlx`, and uses `Int32Builder` and `StringBuilder` to convert the database rows into **Apache Arrow RecordBatches**.
- **Supported Tables**: `orders`, `customers`, `reviews`.

### `src/adapters/mongodb.rs` (MongoDB / NoSQL)
**Purpose**: Maps unstructured BSON documents to flat SQL-ready tables.
- **`register_collection()`**: Connects to MongoDB and uses a cursor to iterate through documents.
- **Schema Mapping**: It explicitly defines a schema (e.g., `item` as `Utf8`, `qty` as `Int32`) so that the SQL engine knows what it's looking at.
- **Supported Collections**: `inventory`, `stock_logs`.

### `src/adapters/s3.rs` & `rest.rs`
**Purpose**: Placeholders and implementation logic for cloud storage and external APIs.
- `s3.rs` registers the local file system as an Object Store, allowing DataFusion to use its native Parquet/CSV reading logic.

---

## ⚡ 3. Engine Orchestration

### `src/main.rs`
**Purpose**: The "Main Loop" and Web Service.
- **Dynamic CSV Scanner**: It uses `std::fs::read_dir("data")` to find all CSV files. For every file it finds, it registers a table named after the file. This is how `users.csv` becomes the table `users`.
- **Query Handler**: This is the logic that receives the SQL string from the browser. It tells DataFusion: "Plan this, Run it, and give me the JSON."
- **JSON Serialization**: It takes the raw binary results from the engine and carefully converts them into a `serde_json` format so the Web UI can display them.

---

## 🎨 4. User Interface Layer

### `index.html`
**Purpose**: The Modern Dashboard.
- **Design**: Uses a "Glassmorphism" design with semi-transparent backgrounds and vibrant gradients to look like a premium, modern software product.
- **Security Logic**: The UI only sends queries to the `/query` endpoint. It does not provide any "Delete" or "Update" buttons, enforcing the project's **Read-Only** security policy.
- **Example Chips**: Pre-loaded SQL queries that demonstrate complex federated joins, making it easy to show off the engine during a demo.

---

## 📁 5. Local Data Layer
### `data/` Directory
- Contains `users.csv`, `products.csv`, `locations.csv`, and `categories.csv`.
- These files are critical for the "CSV Join" examples and are treated as native database tables by the engine.

---

## 🐳 6. Infrastructure
### `Docker`
- The project relies on a `docker-compose.yaml` (or equivalent setup) that runs **PostgreSQL** and **MongoDB** containers. This ensures that the engine is tested against real, production-ready database software rather than simulated mocks.

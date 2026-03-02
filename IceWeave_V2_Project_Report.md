# IceWeave: Professional Federated Query Engine (V2 Final Report)

## 1. Executive Summary
**IceWeave** is a cutting-edge Federated Query Engine designed to unify fragmented data ecosystems. In modern enterprises, data is rarely in one place; it lives across Relational Databases (PostgreSQL), NoSQL Document Stores (MongoDB), and disparate flat files (CSV). IceWeave provides a **single, high-performance SQL interface** to query, filter, and join this data in real-time, eliminating the need for expensive and slow ETL (Extract, Transform, Load) processes.

## 2. Theoretical Framework & Architecture

### 2.1 The Concept of Data Federation
Data Federation is a form of data virtualization where multiple heterogeneous data sources are presented as a single entity. IceWeave implements this by acting as a **Query Orchestrator**. It does not "own" the data; it "manages" the access to it.

### 2.2 V2 Architecture: The In-Memory Snapshot Model
Based on the project's evolution, V2 adopts an **In-Memory MemTable** architecture. 
- **Fetch Phase**: At initialization, IceWeave connect to the various sources (Postgres, Mongo, Local FS).
- **Transformation Phase**: Remote records are converted into the **Apache Arrow** columnar format.
- **Query Phase**: The data is registered into the **Apache DataFusion** session context as virtual tables.
- **Execution Phase**: SQL queries are planned and executed against these in-memory tables at CPU-native speeds.

## 3. Technology Stack (The Full Detailed Shebang)

### 3.1 Infrastructure & Deployment
- **Docker**: The backbone of our development environment. We use Docker to spin up "Enterprise Grade" instances of PostgreSQL and MongoDB instantly, ensuring the engine talks to real network-accessible services.
- **Cargo**: Rust's build system and package manager, ensuring reproducible builds and managed dependencies.

### 3.2 Core Technologies
- **Rust (Programming Language)**: Chosen for its "fearless concurrency" and memory safety without a garbage collector, making it ideal for high-throughput data engines.
- **Apache DataFusion**: A world-class query engine. It provides the SQL parser, logical planner, and physical execution framework.
- **Apache Arrow**: The industry-standard in-memory columnar format. It allows for vectorized operations and zero-copy data sharing.

### 3.3 Critical Rust Crates (The "Brain" of the Engine)
| Crate | Version | Purpose |
| :--- | :--- | :--- |
| `datafusion` | 52.2 | The SQL execution engine and query planner. |
| `arrow` | 52.0+ | Memory management and columnar array builders. |
| `axum` | 0.8 | A high-performance, ergonomic web framework for the backend. |
| `tokio` | 1.0+ | The industry-standard asynchronous runtime for handling concurrent database connections. |
| `sqlx` | 0.8 | A revolutionary async SQL client with compile-time checked queries (used for PostgreSQL). |
| `mongodb` | 3.5 | The official MongoDB driver for Rust, handling BSON-to-Arrow mapping. |
| `serde` / `serde_json` | 1.0 | Handles the data serialization between the Rust backend and the Web UI. |
| `tower-http` | 0.6 | Provides essential web middleware like CORS (for API access) and Static File Serving. |
| `futures` | 0.3 | Essential for managing the "streams" of data coming from NoSQL sources. |
| `rust_decimal` | 1.40 | Ensures 100% accuracy for financial data (Postgres Decimal -> Rust conversion). |

## 4. Federated Data Adapters

### 4.1 The Relational Adapter (PostgreSQL)
Connects via `sqlx` to pull structured schemas. It maps PostgreSQL types (INT, VARCHAR, DECIMAL) directly into Arrow types. 
- **Tables Registered**: `orders`, `customers`, `reviews`.

### 4.2 The NoSQL Adapter (MongoDB)
Handles the challenge of "schema-less" data. It iterates through BSON documents and "flattens" them into a SQL-compatible schema.
- **Collections Registered**: `inventory`, `stock_logs`.

### 4.3 The File System Adapter (CSV)
Implements a dynamic directory crawler. Any file matching `data/*.csv` is automatically linked to the engine.
- **Files Registered**: `users.csv`, `products.csv`, `locations.csv`, `categories.csv`.

## 5. Security & Access Control
IceWeave is designed as a **Read-Query Engine**. 
> [!IMPORTANT]
> **Data Security Constraint**: For administrative security, data manipulation (INSERT, UPDATE, DELETE) is **not allowed** through the IceWeave Web UI. This ensures that the engine remains a "view-only" federated tool. Business logic changes and data mutations must be performed directly through protected Docker shell access or official Database Administration tools (like `psql` or MongoDB Compass).

## 6. Project Impact & Use Case
By joining a CSV of "Products" with a PostgreSQL "Orders" table and a MongoDB "Stock" collection, IceWeave can answer complex business questions in a single query:
*"Which users in the 'South' region bought 'Laptops' that currently have less than 10 items in stock?"*

---
**Status**: V2.0 Final - Presentation Ready.  
**Developed by**: Adarsh  
**Project Objective**: Implementation of a High-Performance Federated Query Engine.

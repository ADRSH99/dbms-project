# IceWeave: Unified Cross-Source Query Processor (V4 Final Project Report)

## 1. Abstract
IceWeave is a high-performance federated query engine that enables the execution of a single Structured Query Language (SQL) query across multiple heterogeneous data sources. By leveraging **Apache DataFusion** for SQL parsing, logical planning, and optimization, and **Apache Arrow** for in-memory data representation, IceWeave provides a unified virtualization layer over relational databases (PostgreSQL), document stores (MongoDB), and flat files (Comma-Separated Values - CSV). The system eliminates the need for complex Extract, Transform, Load (ETL) pipelines by virtualizing data access through a custom federated adapter layer implemented in the Rust programming language. Version 4 introduces dynamic runtime file uploads (SQL scripts, JavaScript scripts, and CSV files) with a seamless engine reload capability, alongside an extensive suite of cross-source join templates.

## 2. Problem Statement
Modern data ecosystems are inherently fragmented. Organizations store structured relational data in PostgreSQL, semi-structured documents in MongoDB, and analytical datasets in CSV or Parquet files. Traditional methods for querying across these systems—such as physical data movement to a Data Warehouse (via ETL pipelines)—introduce significant latency, massive data duplication, and extreme maintenance overhead. There is a critical and immediate need for a processor that can perform "zero-copy" federation, joining data from disparate sources in real-time without moving the physical records until the exact moment of query execution.

## 3. Objectives
- **Unified Structured Query Language Interface**: Provide a standard SQL-92 compliant interface for querying heterogeneous data sources simultaneously.
- **Engine Integration**: Deeply integrate Apache DataFusion as the primary query planning and execution kernel to ensure optimal performance.
- **Custom Adapter Framework**: Develop highly extensible data connectors for PostgreSQL (via the `sqlx` driver), MongoDB (via the Official Rust Driver), and Local File Systems (Dynamic Data Scanner).
- **Dynamic In-Memory Federation**: Implement a high-speed "Snapshot" model in V4 to ensure stable, performant cross-source joins, enhanced with real-time file upload and database re-initialization capabilities without restarting the server.
- **Security by Design**: Implement a strictly read-only query interface via the Web User Interface (UI) to prevent unauthorized data manipulation, enforced through containerization and network isolation.

## 4. System Architecture
IceWeave follows a highly modular, four-layer architecture:
1.  **Web Interface Layer**: A modern, responsive web dashboard (constructed using HyperText Markup Language, Cascading Style Sheets, and JavaScript) for SQL input, file uploads, engine management, and result visualization.
2.  **Application Programming Interface (API) & Orchestration Layer**: Powered by the **Axum** web framework, this layer handles query routing, file management endpoints (upload, delete, list), and communicates asynchronously with the core engine.
3.  **Query Planning & Execution Layer**: Utilizing **Apache DataFusion**, this layer translates inbound SQL text into optimized logical plans and highly efficient physical execution plans.
4.  **Federated Adapter Layer (V4 Dynamic In-Memory Model)**: Connects to external data sources, translates remote data structures into **Apache Arrow RecordBatches**, and registers them as `MemTables` (Memory Tables) within the DataFusion execution context.

## 5. Technical Implementation
IceWeave is built entirely using **Rust** to leverage its industry-leading performance and mathematically proven memory safety (avoiding segmentation faults without garbage collection).
- **Data Ingestion and Reloading**: Upon startup, and dynamically upon user request (`/reload` endpoint), the engine asynchronously connects to PostgreSQL and MongoDB. SQL and JS scripts uploaded by the user are immediately executed using `sqlx::Executor` and the `mongosh` shell to rebuild database schemas on the fly.
- **Schema Mapping**: The engine dynamically maps Binary JSON (BSON) from MongoDB and SQL Rows from PostgreSQL directly to Arrow Columnar Arrays, inferring data types at runtime.
- **Dynamic File Discovery**: A specialized file-system crawler auto-detects, catalogs, and registers any CSV files uploaded to the project's data directory.
- **Execution**: Queries are executed using vectorized Operations (Single Instruction, Multiple Data - SIMD), allowing for complex joins between millions of rows spanning multiple distinct databases in milliseconds, entirely in RAM.

## 6. Detailed Technologies Involved
- **System Language**: Rust (Edition 2024 Framework Compatibility).
- **Primary Frameworks**: 
    - **Apache DataFusion**: The Core Query execution engine, providing the foundation for logical query plans.
    - **Apache Arrow**: The industry-standard in-memory columnar data format, crucial for vectorized operations.
    - **Axum**: A high-performance, ergonomic async web application framework used to build our Representational State Transfer (REST) API backend.
- **Data Source Integrations**:
    - **PostgreSQL Database**: Relational structured storage (PostgreSQL Server Version 15).
    - **MongoDB Database**: NoSQL Document-oriented storage (MongoDB Engine Version 6).
    - **Local CSV Storage**: Standard local file-based dataset storage.
- **Core Rust Libraries (Crates)**:
    - `sqlx`: A purely asynchronous, compile-time verified PostgreSQL connection driver used for database interaction and dynamic script execution.
    - `mongodb`: The official asynchronous MongoDB driver for building connections and reading BSON documents.
    - `tokio`: The industry-standard asynchronous runtime for handling thousands of concurrent database connections and web requests.
    - `serde` and `serde_json`: Essential Serialization and Deserialization libraries for converting complex Rust data structures into JavaScript Object Notation (JSON) for the web client.
    - `tower-http`: Middleware utilities for the Axum web server, providing crucial features like Cross-Origin Resource Sharing (CORS) and static file serving.
    - `axum-multipart`: Specialized libraries for handling asynchronous file uploads from the User Interface directly to the server file system.
- **Containerization and Orchestration**: 
    - **Docker and Docker Compose**: Used to instantly deploy perfectly isolated, deterministic instances of PostgreSQL and MongoDB. Docker guarantees that our backend speaks to real, network-bound enterprise database services rather than mocked data.

## 7. Security Isolation and Docker Architecture
Security and data integrity form the foundation of IceWeave V4, implemented thoroughly at both the software and infrastructure levels:
- **Docker Network Isolation**: The PostgreSQL and MongoDB databases are encapsulated within their own dedicated Docker containers and linked via an internal Docker Bridge Network. They do not expose open ports to the external internet, preventing outside malicious access entirely.
- **Query-Only Application Protocol**: The IceWeave data engine strictly enforces a "read-only" operational model through its Application Programming Interface. Users can completely reconstruct database schemas by uploading authorized `init.sql` and `init.js` scripts through the designated Source Manager, but the SQL Query Input field is completely locked down by DataFusion to only process `SELECT` queries.
- **Prevention of Data Manipulation Language (DML)**: Destructive commands typed into the query editor (such as `DROP TABLE`, `DELETE FROM`, `UPDATE`) are syntactically rejected by the DataFusion parser. Unauthorized data mutation is mathematically impossible through the query Interface. Business logic changes and direct manual modifications must be performed through verified Database Administration tools targeting the secure Docker containers.

## 8. Key Features
- **Dynamic Source Manager**: A massive upgrade in V4 allowing users to upload new PostgreSQL `.sql` initialization scripts, MongoDB `.js` generation scripts, and arbitrary `.csv` files through the browser.
- **Hot-Swappable Engine Reloads**: A single click on "Reload Engine" drops existing database schemas, recreates them using the uploaded scripts, remaps the entire schema into Apache Arrow, and swaps the operational DataFusion Session Context using advanced Thread-Safe Mutex locking (`Arc<RwLock>`) with zero downtime.
- **Cross-Source SQL Joins**: Seamlessly join a CSV file representing users with a PostgreSQL table containing orders and a MongoDB collection mapping inventory.
- **Standardized SQL Support**: Full support for standard mathematical aggregations, filtering (`WHERE` clauses), and inner/outer joins via DataFusion.
- **Premium Web Interface**: Categorized quick-start example queries covering Single Source, Cross-Source Joins, and Multi-Source Federation scenarios for instant demonstration of engine capabilities.

## 9. Expected Outcomes
IceWeave demonstrates a production-grade, highly resilient federated architecture that severely diminishes the complexity and cost of traditional data warehousing for real-time analytics. Version 4 successfully showcases how modern systems can flawlessly combine the raw compute speed of Rust, the analytical power of Apache Arrow, and the infrastructure consistency of Docker to entirely eliminate the pervasive "Data Silo" problem in enterprise environments.

## 10. Conclusion
By aggressively extending Apache DataFusion with a custom federated adapter layer and a state-of-the-art dynamic file handling web interface, IceWeave V4 delivers a practical, extraordinarily scalable solution for cross-source querying. It fundamentally proves that a fully unified view of highly heterogeneous organizational data can be robustly achieved without physical data replication, accelerating analytical insights while heavily minimizing infrastructure footprint and maintenance costs.

---
**Status**: V4.0 Final Release  
**Architecture Theme**: Dynamic In-Memory Federation with Runtime Hot-Reloading  
**Developed by**: Adarsh Bellamane, KV Akash, Anirudh Trichy  
**Date**: March 2026

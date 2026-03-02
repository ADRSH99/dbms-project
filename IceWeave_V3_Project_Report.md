# IceWeave: Unified Cross-Source Query Processor (V3 Final Project Report)

## 1. Abstract
IceWeave is a high-performance federated query engine that enables the execution of a single SQL query across multiple heterogeneous data sources. By leveraging **Apache DataFusion** for SQL parsing, logical planning, and optimization, and **Apache Arrow** for in-memory data representation, IceWeave provides a unified abstraction layer over relational databases (PostgreSQL), document stores (MongoDB), and flat files (CSV). The system eliminates the need for complex ETL pipelines by virtualizing data access through a custom federated adapter layer implemented in Rust.

## 2. Problem Statement
Modern data ecosystems are inherently fragmented. Organizations store structured data in PostgreSQL, semi-structured documents in MongoDB, and analytical datasets in CSV or Parquet files. Traditional methods for querying across these systems—such as physical data movement to a Data Warehouse (ETL)—introduce significant latency, data duplication, and maintenance overhead. There is a critical need for a processor that can perform "zero-copy" federation, joining data from disparate sources in real-time without moving the physical records until query time.

## 3. Objectives
- **Unified SQL Interface**: Provide a standard SQL-92 compliant interface for querying heterogeneous sources.
- **Engine Integration**: Deeply integrate Apache DataFusion as the primary query planning and execution kernel.
- **Custom Adapter Framework**: Develop extensible connectors for PostgreSQL (SQLx), MongoDB (Official Driver), and Local FS (Dynamic Scanner).
- **In-Memory Federation**: Implement a high-speed "Snapshot" model in V2 to ensure stable and performant cross-source joins.
- **Security by Design**: Implement a read-only query interface via the Web UI to prevent unauthorized data manipulation.

## 4. System Architecture
IceWeave follows a modular, four-layer architecture:
1.  **Web Interface Layer**: A modern, responsive dashboard (HTML/CSS/JS) for SQL input and result visualization.
2.  **API & Orchestration Layer**: Powered by **Axum**, this layer handles query routing and communicates with the engine.
3.  **Query Planning & Execution Layer**: Utilizing **Apache DataFusion**, it translates SQL into optimized logical and physical plans.
4.  **Federated Adapter Layer (V2 In-Memory)**: Translates remote data into **Apache Arrow RecordBatches** and registers them as `MemTables` within the execution context.

## 5. Technical Implementation
IceWeave is built using **Rust** for its industry-leading performance and memory safety. 
- **Data Ingestion**: Upon startup, the engine asynchronously connects to PostgreSQL and MongoDB.
- **Schema Mapping**: The engine dynamically maps BSON (Mongo) and SQL Rows (Postgres) to Arrow Columnar Arrays.
- **Dynamic File Discovery**: A specialized file-system crawler auto-detects any CSV files in the project's data directory.
- **Execution**: Queries are executed using vectorized operations, allowing for joins between millions of rows in milliseconds once loaded into memory.

## 6. Technologies Involved
- **Programming Language**: Rust (Edition 2021)
- **Primary Frameworks**: 
    - **Apache DataFusion**: Core Query Engine.
    - **Apache Arrow**: In-memory Columnar Format.
    - **Axum**: Web Framework for Backend API.
- **Data Sources**:
    - **PostgreSQL**: Relational storage (running in Docker).
    - **MongoDB**: NoSQL Document storage (running in Docker).
    - **CSV**: Local file-based storage.
- **Key Rust Crates**:
    - `sqlx`: Async PostgreSQL driver.
    - `mongodb`: Official MongoDB driver.
    - `tokio`: Async runtime.
    - `serde`: Serialization/Deserialization.
- **Containerization**: Docker (Docker Desktop/Engine) for database isolation.

## 7. Key Features
- **Cross-Source SQL Joins**: Seamlessly join a CSV file with a Postgres table and a Mongo collection.
- **Dynamic CSV Registration**: Zero-config addition of new data files.
- **Standardized SQL Support**: Full support for filters, aggregations, and complex joins via DataFusion.
- **Premium Web UI**: Responsive "Ice" themed dashboard with example query templates.
- **Snapshot Stability**: V2 architecture ensures reliability across disparate database versions.

## 8. Expected Outcomes
IceWeave demonstrates a production-style federated architecture that avoids the complexity of data warehousing for real-time analytics. The project successfully showcases how modern systems can combine the speed of Rust with the power of Apache Arrow to solve the "Data Silo" problem.

## 9. Project Security Narrative
**Query-Only Protocol**: To ensure the integrity of source systems, the IceWeave engine is strictly limited to `SELECT` operations via its web interface. Data manipulation (INSERT, UPDATE, DELETE) is intentionally blocked at the API level to prevent unauthorized mutations. All data modifications must occur directly through the authorized database containers using professional administration tools (psql, mongosh).

## 10. Conclusion
By extending Apache DataFusion with a custom federated adapter layer, IceWeave provides a practical, scalable solution for cross-source querying. It proves that a unified view of organizational data can be achieved without data replication, fostering faster insights and lower infrastructure costs.

---
**Status**: V3.0 Final Release  
**Developed by**: Adarsh Bellamane, KV Akash, Anirudh Trichy
**Date**: March 2026

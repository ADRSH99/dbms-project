# IceWeave: Federated Query Engine (V4)

IceWeave is a high-performance, in-memory federated query engine built in Rust. It enables users to execute single SQL queries that transparently join data across PostgreSQL, MongoDB, and local CSV files. Version 4 introduces a dynamic Data Source Manager, allowing runtime uploads of initialization scripts and datasets, with instant hot-swappable engine reloads.

## 🚀 Key Features
- **Zero-Copy Federation**: Query across relational, document, and flat-file data sources simultaneously without ETL pipelines.
- **Dynamic Hot-Reloading (V4)**: Upload new `.sql` scripts, `.js` scripts, and `.csv` files via the browser and instantly rebuild the in-memory engine without restarting the server.
- **Blazing Fast**: Powered by Apache DataFusion and Apache Arrow for vectorized, in-memory execution.
- **Strict Security Isolation**: Designed as a read-only query layer. All underlying databases are strictly isolated within Docker container networks.
- **Premium User Interface**: Features a glassmorphism dashboard with 15 pre-configured example queries spanning Single Source, Cross-Source, and Full Federation scenarios.

## 🛠️ Technology Stack
- **Core Engine**: Rust, Apache DataFusion, Apache Arrow
- **Web Backend**: Axum, Tokio, Tower-HTTP, axum-multipart
- **Database Adapters**: `sqlx` (PostgreSQL), `mongodb` (Official Rust Driver)
- **Infrastructure**: Docker, Docker Compose

## 📦 Running the Project

1. **Start the Database Containers** (Postgres and Mongo):
   ```bash
   # Ensure Docker is running
   docker compose up -d
   ```

2. **Run the IceWeave Engine**:
   ```bash
   cargo run
   ```

3. **Access the Dashboard**:
   Open your browser and navigate to: `http://localhost:3000`

## 🛡️ Security & Architecture Structure
IceWeave relies heavily on **Docker** to enforce mathematical network and storage isolation. The database instances (PostgreSQL and MongoDB) are spun up inside isolated Linux containers on a dedicated virtual bridge network. They do not expose operational ports to the internet. The Rust Axum server acts as the only gateway, firmly establishing a read-only protocol where Data Management Language (DML - like `DROP` or `UPDATE`) is syntactically stripped by DataFusion parser before execution. Modifications to the structural schema must be done intentionally through the authorized "Upload & Reload" workflow in the V4 User Interface.

*For full technical details, refer to `IceWeave_V4_Project_Report.md` and `Explanation.md`.*

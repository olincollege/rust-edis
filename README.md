# Rust-EDIS: Scalable Reader/Writer Shard Architecture

## Overview

**Rust-EDIS** implements a scalable, distributed key-value store based on the **Reader/Writer Shard Model**, inspired by Redis. The system enables efficient data storage, fault tolerance, and high scalability, making it ideal for applications requiring dynamic scaling and consistent performance.

## Features

- **Write Shards:** Isolated shards handle key-value pairs using deterministic hashing to maintain consistency.
- **Read Shards:** Synchronize with write shards to replicate data, enabling eventual consistency. Clients access read replicas through a round-robin strategy.
- **Dynamic Shard Management:** Add read replicas dynamically for increased fault tolerance and scalability.

## Architecture

1. **Main Info Instance:**
   - Tracks the IP/ports of all shards (read/write).
   - Assists with shard discovery and management.

2. **Write Shards:**
   - Store key-value pairs.
   - Deterministic hashing ensures a specific shard handles a given key.

3. **Read Shards:**
   - Maintain replicas of data from write shards.
   - Round-robin distribution ensures load balancing for read operations.

4. **Client Binary:**
   - Provides commands for reading and writing data.
   - Handles shard computation and request routing.

## Project Structure

- `src/client.rs`: Implements the client for interacting with the system.
- `src/info.rs`: Manages shard discovery and dynamic configuration.
- `src/write_shard.rs`: Handles write requests and key-value storage.
- `src/read_shard.rs`: Synchronizes with write shards and processes read requests.

## Prerequisites

- **Rust Toolchain:** Ensure you have Rust installed. You can download it from [Rust's official site](https://www.rust-lang.org/tools/install).

## Setup and Usage

### Step 1: Clone the Repository

```bash
git clone https://github.com/your-repo-name.git
cd rust-edis
```

### Step 2: Build the Project

```bash
cargo build
```

### Step 3: Run the Components

1. **Start the `info` instance:**

   ```bash
   cargo run --bin info
   ```

2. **Start a write shard:**

   ```bash
   cargo run --bin write_shard
   ```

3. **Start a read shard:**

   ```bash
   cargo run --bin read_shard
   ```

4. **Run the client:**

   ```bash
   cargo run --bin client
   ```

### Step 4: Interact with the System

- **Write Data:** Use the client binary to send write requests.
- **Read Data:** The client binary sends read requests, selects a read shard, and retrieves the data.

## Dependencies

For the full dependency list, refer to the `[dependencies]` section in `Cargo.toml`.

## Testing

To run the tests:

```bash
cargo test
```

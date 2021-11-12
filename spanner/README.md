# google-cloud-spanner

Google Cloud Platform GCE spanner library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-spanner.svg)](https://crates.io/crates/google-cloud-spanner)

* [About Cloud Spanner](https://cloud.google.com/spanner/)
* [API Documentation](https://cloud.google.com/spanner/docs)
* [Rust client Documentation](./README.md#Documentation)

## Installation

```
[dependencies]
google-cloud-spanner = 0.1.0
```

## Quick Start

Create `Client` and call transaction API same as [Google Cloud Go](https://github.com/googleapis/google-cloud-go/tree/main/spanner).

```rust
use google_cloud_spanner::client::Client;

#[tokio::main]
async fn main() {

    const DATABASE: &str = "projects/your_projects/instances/your-instance/databases/your-database";
   
    // Create spanner client
    let mut client = match Client::new(DATABASE, None).await {
        Ok(client) => client,
        Err(e) => { /* handle error */ }
    };
    
    //Reading transactions.
    client.single(); 
    client.read_only_transaction(); 
    client.batch_read_only_transaction();

    //Reading and writing transactions.
    client.apply();
    client.read_write_transaction();
    client.apply_at_least_once();
    client.partitioned_update();
    
    //close  
    client.close();
}
```

## Documentation

### Overview
* [Creating a Client](./README.md#Creating a Client)
* [Simple Reads and Writes](./README.md#Simple Reads and Writes)
* [Keys](./README.md#Keys)
* [KeyRanges](./README.md#KeyRanges)
* [KeySets](./README.md#KeySets)
* [Transactions](./README.md#Transactions)
* [Single Reads](./README.md#Single Reads)
* [Statements](./README.md#Statements)
* [Rows](./README.md#Rows)
* [Multiple Reads](./README.md#Multiple Reads)
* [Timestamps and Timestamp Bounds](./README.md#Timestamps and Timestamp Bounds)
* [Mutations](./README.md#Mutations)
* [Writes](./README.md#Writes)
* [Structs](./README.md#Structs)
* [DML and Partitioned DML](./README.md#DML and Partitioned DML)

Package spanner provides a client for reading and writing to Cloud Spanner databases.   
See the packages under admin for clients that operate on databases and instances.

### Creating a Client

To start working with this package, create a client that refers to the database of interest:

```rust
const DATABASE: &str = "projects/your_projects/instances/your-instance/databases/your-database";
let mut client = match Client::new(DATABASE, None).await {
    Ok(client) => client,
    Err(e) => { /* handle error */ }
};

client.close();
```

Remember to close the client after use to free up the sessions in the session pool.  
  
To use an emulator with this library, you can set the SPANNER_EMULATOR_HOST environment variable to the address at which your emulator is running. This will send requests to that address instead of to Cloud Spanner.   You can then create and use a client as usual:

```rust

// Set SPANNER_EMULATOR_HOST environment variable.
std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");

// Create client as usual.
const DATABASE: &str = "projects/your_projects/instances/your-instance/databases/your-database";
let mut client = match Client::new(DATABASE, None).await {
    Ok(client) => client,
    Err(e) => { /* handle error */ }
};
```

### Simple Reads and Writes
Two Client methods, Apply and Single, work well for simple reads and writes. As a quick introduction, here we write a new row to the database and read it back:
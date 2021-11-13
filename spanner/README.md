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

## <a name="Documentation"></a>Documentation

### Overview
* [Creating a Client](#CreatingAClient)
* [Simple Reads and Writes](./README.md#Simple%20Reads%20and%20Writes)
* [Keys](./README.md#Keys)
* [KeyRanges](./README.md#KeyRanges)
* [KeySets](./README.md#KeySets)
* [Transactions](./README.md#Transactions)
* [Single Reads](./README.md#Single%20Reads)
* [Statements](./README.md#Statements)
* [Rows](./README.md#Rows)
* [Multiple Reads](./README.md#Multiple%20Reads)
* [Timestamps and Timestamp Bounds](./README.md#Timestamps%20and%20Timestamp Bounds)
* [Mutations](./README.md#Mutations)
* [Writes](./README.md#Writes)
* [Structs](./README.md#Structs)
* [DML and Partitioned DML](./README.md#DML%20and%20Partitioned%20DML)

Package spanner provides a client for reading and writing to Cloud Spanner databases.   
See the packages under admin for clients that operate on databases and instances.

### <a name="CreatingAClient"></a>Creating a Client

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
use google_cloud_spanner::client::Client;

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

```rust
use google_cloud_spanner::{mutation,value,key};

let mutation = mutation::insert("User",
    vec!["UserID", "Name", "UpdatedAt"], // columns 
    vec![1.to_kind(), "name".to_kind(), value::CommitTimestamp::new().to_kind()]
);
let commit_timestamp = client.apply(vec![mutation],None).await?;

let row = client.single(None).await?.read("User",
    vec!["UserID", "Name", "UpdatedAt"],
    key::Key::one(1), 
    None
).await?;
```

All the methods used above are discussed in more detail below.

### Keys

Every Cloud Spanner row has a unique key, composed of one or more columns. Construct keys with a literal of type Key:

```rust
use google_cloud_spanner::{key};

let key1 = key::Key::one("key");
```

### KeyRanges

The keys of a Cloud Spanner table are ordered. You can specify ranges of keys using the KeyRange type:

```rust
use google_cloud_spanner::key::{Key,KeyRange,RangeKind};

let start = Key::one(1);
let end = Key::one(100);
let range1 = KeyRange::new(start, end, RangeKind::ClosedClosed);
let range2 = KeyRange::new(start, end, RangeKind::ClosedOpen);
let range3 = KeyRange::new(start, end, RangeKind::OpenOpen);
let range4 = KeyRange::new(start, end, RangeKind::OpenClosed);
```

### KeySets

A KeySet represents a set of keys. A single Key or KeyRange can act as a KeySet.

```rust
use google_cloud_spanner::key::Key;

let key1 = Key::new(vec!["Bob".to_kind(), "2014-09-23".to_kind()]);
let key2 = Key::new(vec!["Alfred".to_kind(), "2015-06-12".to_kind()]);
let ks = vec![key1,key2] ;
let rows = tx.read("Table", vec!["Name","BirthDay"], ks, None).await;
```

all_keys returns a KeySet that refers to all the keys in a table:

```rust
use google_cloud_spanner::key::all_keys;

let ks = all_keys();
```

### Transactions

All Cloud Spanner reads and writes occur inside transactions. There are two types of transactions, read-only and read-write. Read-only transactions cannot change the database, do not acquire locks, and may access either the current database state or states in the past. Read-write transactions can read the database before writing to it, and always apply to the most recent database state.

### Single Reads
The simplest and fastest transaction is a ReadOnlyTransaction that supports a single read operation. Use Client.Single to create such a transaction. You can chain the call to Single with a call to a Read method.

When you only want one row whose key you know, use ReadRow. Provide the table name, key, and the columns you want to read:

```rust
let row = client.single(None).await?.read_row("Table", vec!["col1", "col2"], key::Key::one(1)).await?;
```

Read multiple rows with the Read method. It takes a table name, KeySet, and list of columns:

```rust
let iter = client.single(None).await?.read("Table", vec!["col1", "col2"], key::Key::one(1), None).await?;
```

Read returns a RowIterator. You can call the Do method on the iterator and pass a callback:

```
TODO 
```

RowIterator also follows the standard pattern for the Google Cloud Client Libraries:

```
loop {
    let row = match iter.next().await? {
        Some(row) => row,
        None => break,
    };
    
    //TODO: use row
};
```

* The used session is returned to the drop timing session pool, so unlike Go, there is no need to call Stop.  

* To read rows with an index, use `ReadOptions`.

### Statements

The most general form of reading uses SQL statements. Construct a Statement with NewStatement, setting any parameters using the Statement's Params map:

```rust
use google_cloud_spanner::statement::Statement;

let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
stmt.add_param("UserId", user_id);
```

You can also construct a Statement directly with a struct literal, providing your own map of parameters.

Use the Query method to run the statement and obtain an iterator:

```rust
let iter = client.single(None).await?.query(stmt, None).await?;
```

### Rows
Once you have a Row, via an iterator or a call to ReadRow, you can extract column values in several ways. Pass in a pointer to a Go variable of the appropriate type when you extract a value.  

You can extract by column position or name:

```
let value           = row.column::<String>(0)?;
let nullable_value  = row.column::<Option<String>>(1)?;
let array_value     = row.column_by_name::<Vec<i64>>("array")?;
let struct_data     = row.column_by_name::<Vec<TestStruct>>("struct_data")?;
```

Or you can define a Rust struct that corresponds to your columns, and extract into that:
* `TryFromStruct` trait is required

```
struct TestStruct {
    pub struct_field: String,
    pub struct_field_time: NaiveDateTime,
    pub commit_timestamp: CommitTimestamp,
}

impl TryFromStruct for TestStruct {
    fn try_from(s: RowStruct<'_>) -> Result<Self> {
        Ok(TestStruct {
            struct_field: s.column_by_name("struct_field")?,
            struct_field_time: s.column_by_name("struct_field_time")?,
            commit_timestamp: s.column_by_name("commit_timestamp")?,
        })
    }
}
```

### Multiple Reads

To perform more than one read in a transaction, use ReadOnlyTransaction:

```rust
let txn = client.read_only_transaction(None).await?;
let iter1 = txn.query(ctx, stmt1, None).await;
// ...
let iter2 =  txn.query(ctx, stmt2, None).await;
// ...
```

* The used session is returned to the drop timing session pool, so unlike Go, there is no need to call txn Close.

### Timestamps and Timestamp Bounds

Cloud Spanner read-only transactions conceptually perform all their reads at a single moment in time, called the transaction's read timestamp. Once a read has started, you can call ReadOnlyTransaction's Timestamp method to obtain the read timestamp.

By default, a transaction will pick the most recent time (a time where all previously committed transactions are visible) for its reads. This provides the freshest data, but may involve some delay. You can often get a quicker response if you are willing to tolerate "stale" data. You can control the read timestamp selected by a transaction by calling the WithTimestampBound method on the transaction before using it. For example, to perform a query on data that is at most one minute stale, use
```
TODO
```
See the documentation of TimestampBound for more details.

### Mutations

To write values to a Cloud Spanner database, construct a Mutation. The spanner package has functions for inserting, updating and deleting rows. Except for the Delete methods, which take a Key or KeyRange, each mutation-building function comes in three varieties.

One takes lists of columns and values along with the table name:

```
use google_cloud_spanner::{mutation,value,key};

let mutation = mutation::insert("User",
    vec!["UserID", "Name", "UpdatedAt"], // columns 
    vec![1.to_kind(), "name".to_kind(), value::CommitTimestamp::new().to_kind()]
);
```

One takes a map from column names to values:

```
TODO 
```

And the third accepts a struct value, and determines the columns from the struct field names:

```rust
struct TestStruct {
        pub struct_field: String,
        pub struct_field_time: NaiveDateTime,
        pub commit_timestamp: CommitTimestamp,
    }

impl ToStruct for TestStruct {
    fn to_kinds(&self) -> Kinds {
        vec![
            ("struct_field", self.struct_field.to_kind()),
            ("struct_field_time", self.struct_field_time.to_kind()),
            ("commit_timestamp",NaiveDateTime::from(self.commit_timestamp).to_kind()),
        ]
    }

    fn get_types() -> Types {
        vec![
            ("struct_field", String::get_type()),
            ("struct_field_time", NaiveDateTime::get_type()),
            ("commit_timestamp", CommitTimestamp::get_type()),
        ]
    }
}

TODO InsertStruct
```

### Writes

To apply a list of mutations to the database, use Apply:
```rust
use google_cloud_spanner::{mutation,key};

let m1 = mutation::delete("Table", key::all_keys());
let m2 = mutation::insert("Table", key::all_keys());
let commit_timestamp = client.apply(vec![m1,m2],None).await?;
```

If you need to read before writing in a single transaction, use a ReadWriteTransaction. ReadWriteTransactions may be aborted automatically by the backend and need to be retried. You pass in a function to ReadWriteTransaction, and the client will handle the retries automatically. Use the transaction's BufferWrite method to buffer mutations, which will all be executed at the end of the transaction:

```rust
 let (commit_timestamp, row) = client.read_write_transaction(
    |mut tx| async move {
        let result = async {

            // The buffered mutation will be committed.  If the commit
            // fails with an Aborted error, this function will be called again.
            let m1 = mutation::insert("User", vec![], vec![1.to_kind(), CommitTimestamp::new().to_kind()])
            let m2 = mutation::insert("User", vec![], vec![2.to_kind(), CommitTimestamp::new().to_kind()])
            tx.buffer_write(vec![m1,m2]);

            // The transaction function will be called again if the error code
            // of this error is Aborted. The backend may automatically abort
            // any read/write transaction if it detects a deadlock or other problems.
            tx.read_row("User", vec!["UserId"], vec![Key::one(*user_id_ref.clone())], None).await
        }
        .await;
        //return owner ship of read_write_transaction
        (tx, result)
    },
    None
).await?;
```
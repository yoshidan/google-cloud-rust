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

    const DATABASE: &str = "projects/your_project/instances/your-instance/databases/your-database";
   
    // Create spanner client
    let mut client = match Client::new(DATABASE).await {
        Ok(client) => client,
        Err(e) => { /* handle error */ }
    };
}
```

## Example
Here is the example with using Warp.
* https://github.com/yoshidan/google-cloud-rust-example/tree/main/spanner/rust

## Performance 

Result of the 24 hours Load Test.

| Metrics | This library | Google Cloud Go | 
| -------- | ----------------| ----------------- |
| RPS | [439.5](https://storage.googleapis.com/0432808zbaeatxa/report_1637587806.875014.html) |  |
| Used vCPU | [0.35~0.38](https://storage.googleapis.com/0432808zbaeatxa/CPU%20(3).png) |  |

* [Rust report](https://storage.googleapis.com/0432808zbaeatxa/report_1637587806.875014.html)
* [Go report](.)

Test Condition 
* 2.0 vCPU limited GKE Autopilot Pod
* 1 Node spanner database server
* 100 Users
* [Here](https://github.com/yoshidan/google-cloud-rust-example/tree/main/spanner) is the application for Load Test.

## <a name="Documentation"></a>Documentation

### Overview
* [Creating a Client](#CreatingAClient)
* [Simple Reads and Writes](#SimpleReadsAndWrites)
* [Keys](./README.md#Keys)
* [KeyRanges](./README.md#KeyRanges)
* [KeySets](./README.md#KeySets)
* [Transactions](./README.md#Transactions)
* [Single Reads](#SingleReads)
* [Statements](./README.md#Statements)
* [Rows](./README.md#Rows)
* [Multiple Reads](#MultipleReads)
* [Timestamps and Timestamp Bounds](#TimestampsAndTimestampBounds)
* [Mutations](./README.md#Mutations)
* [Writes](./README.md#Writes)
* [Structs](./README.md#Structs)
* [DML and Partitioned DML](#DMLAndPartitionedDML)

Package spanner provides a client for reading and writing to Cloud Spanner databases.   
See the packages under admin for clients that operate on databases and instances.

### <a name="CreatingAClient"></a>Creating a Client

To start working with this package, create a client that refers to the database of interest:

```rust
const DATABASE: &str = "projects/your_projects/instances/your-instance/databases/your-database";
let mut client = match Client::new(DATABASE).await {
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
let mut client = match Client::new(DATABASE).await {
    Ok(client) => client,
    Err(e) => { /* handle error */ }
};
```

### <a name="SimpleReadsAndWrites"></a>Simple Reads and Writes
Two Client methods, Apply and Single, work well for simple reads and writes. As a quick introduction, here we write a new row to the database and read it back:

```rust
use google_cloud_spanner::mutation::insert;
use google_cloud_spanner::key::Key;
use google_cloud_spanner::value::CommitTimestamp;

let mutation = insert("User",
    vec!["UserID", "Name", "UpdatedAt"], // columns 
    vec![1.to_kind(), "name".to_kind(), CommitTimestamp::new().to_kind()]
);
let commit_timestamp = client.apply(vec![mutation]).await?;

let row = client.single().await?.read(
    "User",
    vec!["UserID", "Name", "UpdatedAt"],
    Key::one(1), 
).await?;
```

All the methods used above are discussed in more detail below.

### Keys

Every Cloud Spanner row has a unique key, composed of one or more columns. Construct keys with a literal of type Key:

```rust
use google_cloud_spanner::key::Key;

let key1 = Key::one("key");
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
let rows = tx.read("Table", vec!["Name","BirthDay"], ks).await;
```

all_keys returns a KeySet that refers to all the keys in a table:

```rust
use google_cloud_spanner::key::all_keys;

let ks = all_keys();
```

### Transactions

All Cloud Spanner reads and writes occur inside transactions. There are two types of transactions, read-only and read-write. Read-only transactions cannot change the database, do not acquire locks, and may access either the current database state or states in the past. Read-write transactions can read the database before writing to it, and always apply to the most recent database state.

### <a name="SingleReads"></a>Single Reads
The simplest and fastest transaction is a ReadOnlyTransaction that supports a single read operation. Use Client.Single to create such a transaction. You can chain the call to Single with a call to a Read method.

When you only want one row whose key you know, use ReadRow. Provide the table name, key, and the columns you want to read:

```rust
let row = client.single().await?.read_row("Table", vec!["col1", "col2"], Key::one(1)).await?;
```

Read multiple rows with the Read method. It takes a table name, KeySet, and list of columns:

```rust
let iter1 = client.single().await?.read("Table", vec!["col1", "col2"], vec![
    Key::one(1), 
    Key::one(2)
]).await?;

let iter2 = client.single().await?.read("Table", vec!["col1", "col2"], vec![
    Key::new(vec!["composite1-1".to_kind(),"composite1-2".to_kind()]),
    Key::new(vec!["composite2-1".to_kind(),"composite2-1".to_kind()])
]).await?;
```

RowIterator also follows the standard pattern for the Google Cloud Client Libraries:

```rust
loop {
    let row = match iter.next().await? {
        Some(row) => row,
        None => break,
    };
    
    //TODO: use row
};
```

* The used session is returned to the drop timing session pool, so unlike Go, there is no need to call Stop.  

* To read rows with an index, use `client.read_with_option`.

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
let iter = client.single().await?.query(stmt).await?;
```

### Rows
Once you have a Row, via an iterator or a call to read_row, you can extract column values in several ways. Pass in a pointer to a Rust variable of the appropriate type when you extract a value.  

You can extract by column position or name:

```rust
let value           = row.column::<String>(0)?;
let nullable_value  = row.column::<Option<String>>(1)?;
let array_value     = row.column_by_name::<Vec<i64>>("array")?;
let struct_data     = row.column_by_name::<Vec<User>>("struct_data")?;
```

Or you can define a Rust struct that corresponds to your columns, and extract into that:
* `TryFromStruct` trait is required

```rust
pub struct User {
    pub user_id: String,
    pub premium: bool,
    pub updated_at: DateTime<Utc>
}

impl TryFromStruct for User {
    fn try_from(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(User {
            user_id: s.column_by_name("UserId")?,
            premium: s.column_by_name("Premium")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}
```

### <a name="MultipleReads"></a>Multiple Reads

To perform more than one read in a transaction, use ReadOnlyTransaction:

```rust
let tx = client.read_only_transaction().await?;

let mut stmt = Statement::new("SELECT * , \
            ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
            ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
            FROM User \
            WHERE UserId = @Param1");

stmt.add_param("Param1", user_id);
let mut reader = tx.query(stmt).await?;
loop {
    let row = match reader.next().await.map_err(|e| Error::GRPC(e))?{
        Some(row) => row,
        None => println!("end of record")
    };
    let user_id= row.column_by_name::<String>("UserId")?
    let user_items= row.column_by_name::<Vec<model::UserItem>>("UserItem")?
    let user_characters = row.column_by_name::<Vec<model::UserCharacter>>("UserCharacter")?
    data.push(user_id);
}

let reader2 = tx.read("User", vec!["UserID"], vec![
    Key::new(vec!["composite1-1".to_kind(),"composite1-2".to_kind()]),
    Key::new(vec!["composite2-1".to_kind(),"composite2-1".to_kind()])
]).await?;

// ...

};
```

* The used session is returned to the drop timing session pool, so unlike Go, there is no need to call txn Close.

### <a name="TimestampsAndTimestampBounds"></a>Timestamps and Timestamp Bounds

Cloud Spanner read-only transactions conceptually perform all their reads at a single moment in time, called the transaction's read timestamp. Once a read has started, you can call ReadOnlyTransaction's Timestamp method to obtain the read timestamp.

By default, a transaction will pick the most recent time (a time where all previously committed transactions are visible) for its reads. This provides the freshest data, but may involve some delay. You can often get a quicker response if you are willing to tolerate "stale" data.   
You can control the read timestamp selected by a transaction. For example, to perform a query on data that is at most one minute stale, use

```rust 
use google_cloud_spanner::value::TimestampBound;

let tx = client.single(TimestampBound::max_staleness(Duration::from_secs(60)).await?;
```

See the documentation of TimestampBound for more details.

### Mutations

To write values to a Cloud Spanner database, construct a Mutation. The spanner package has functions for inserting, updating and deleting rows. Except for the Delete methods, which take a Key or KeyRange, each mutation-building function comes in three varieties.

One takes lists of columns and values along with the table name:

```rust
use google_cloud_spanner::mutation::insert;
use google_cloud_spanner::value::CommitTimestamp;

let mutation = insert("User",
    vec!["UserID", "Name", "UpdatedAt"], // columns 
    vec![1.to_kind(), "name".to_kind(), CommitTimestamp::new().to_kind()] // values
);
```

And the third accepts a struct value, and determines the columns from the struct field names:

* `ToStruct` trait is required

```rust
pub struct User {
    pub user_id: String,
    pub premium: bool,
    pub updated_at: DateTime<Utc>
}

impl ToStruct for User {
    fn to_kinds(&self) -> Kinds {
        vec![
            ("UserId", self.user_id.to_kind()),
            ("Premium", self.premium.to_kind()),
            ("UpdatedAt", CommitTimestamp::new().to_kind())
        ]
    }

    fn get_types() -> Types {
        vec![
            ("UserId", String::get_type()),
            ("Premium", bool::get_type()),
            ("UpdatedAt", CommitTimestamp::get_type())
        ]
    }
}
```

```rust
use uuid::Uuid;
use google_cloud_spanner::mutation::insert_struct;

let new_user = model::User {
    user_id: Uuid::new_v4().to_string(),
    premium: true,
    updated_at: Utc::now(),
};
let new_user2 = model::User {
    user_id: Uuid::new_v4().to_string(),
    premium: false,
    updated_at: Utc::now(),
};
let m1 = insert_or_update_struct("User", new_user);
let m2 = insert_or_update_struct("User", new_user2);
```

### Writes

To apply a list of mutations to the database, use Apply:
```rust
use google_cloud_spanner::mutation::insert;
use google_cloud_spanner::mutation::delete;
use google_cloud_spanner::key::all_keys;

let m1 = delete("Table", all_keys());
let m2 = insert("Table", all_keys());
let commit_timestamp = client.apply(vec![m1,m2]).await?;
```

If you need to read before writing in a single transaction, use a ReadWriteTransaction. ReadWriteTransactions may be aborted automatically by the backend and need to be retried. You pass in a function to ReadWriteTransaction, and the client will handle the retries automatically. Use the transaction's BufferWrite method to buffer mutations, which will all be executed at the end of the transaction:

```rust
use google_cloud_spanner::client::TxError;
use google_cloud_spanner::mutation;
use google_cloud_spanner::key::Key;

let tx_result: Result<(Option<Timestamp>,()), Error> = client.read_write_transaction(|mut tx| async {
    // The transaction function will be called again if the error code
    // of this error is Aborted. The backend may automatically abort
    // any read/write transaction if it detects a deadlock or other problems.
    let result: Result<(), Error> = async {
        let mut reader = tx.read("UserItem", vec!["UserId", "ItemId", "Quantity"], Key::one(user_id.to_string())).await?;
        let ms  = loop {
            let mut ms = vec![];
            let row = reader.next().await?;
            match row {
                Some(row) => {
                    let item_id = row.column_by_name::<i64>("ItemId")?;
                    let quantity = row.column_by_name::<i64>("Quantity")?;
                    ms.push(update("UserItem", vec!["Quantity"], vec![
                        user_id.to_string().to_kind(),
                        item_id.to_kind(),
                        (quantity + 1).to_kind(),
                    ]));
                },
                None => break ms
            }
        };

        // The buffered mutation will be committed.  If the commit
        // fails with an Aborted error, this function will be called again
        tx.buffer_write(ms);
        Ok(())
    }.await;

    //return owner ship of read_write_transaction
    (tx, result)
}).await;
```

The Error of the `read_write_transaction` must implements 
* From<google_cloud_googleapis::Status>
* From<google_cloud_spanner::session::SessionError>
* google_cloud_gax::TryAs<google_cloud_googleapis::Status>

```rust
#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    ParseError(#[from] google_cloud_spanner::row::Error),
    #[error(transparent)]
    GRPC(#[from] google_cloud_googleapis::Status),
    #[error(transparent)]
    SessionError(#[from] google_cloud_spanner::session::SessionError),
}

impl TryAs<Status> for Error {
    fn try_as(&self) -> Result<&Status,()> {
        match self {
            Error::GRPC(s) => Ok(s),
            _ => Err(())
        }
    }
}
```

### <a name="DMLAndPartitionedDML"></a>DML and Partitioned DML
Spanner supports DML statements like INSERT, UPDATE and DELETE. Use ReadWriteTransaction.Update to run DML statements. It returns the number of rows affected. (You can call use ReadWriteTransaction.Query with a DML statement. The first call to Next on the resulting RowIterator will return iterator.Done, and the RowCount field of the iterator will hold the number of affected rows.)

For large databases, it may be more efficient to partition the DML statement. Use client.PartitionedUpdate to run a DML statement in this way. Not all DML statements can be partitioned.

```rust
let client = Client::new(DATABASE).await.context("error")?;
let stmt = Statement::new("UPDATE User SET Value = 'aaa' WHERE Value IS NOT NULL");
let result = client.partitioned_update(stmt).await?;
```
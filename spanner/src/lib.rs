//! # google-cloud-spanner
//!
//! Google Cloud Platform spanner library.
//!
//! * [About Cloud Spanner](https://cloud.google.com/spanner/)
//! * [Spanner API Documentation](https://cloud.google.com/spanner/docs)
//! * [Rust client Documentation](#Documentation)
//! ## Quick Start
//!
//! Create `Client` and call transaction API same as [Google Cloud Go](https://github.com/googleapis/google-cloud-go/tree/main/spanner).
//!
//! ```
//! use google_cloud_spanner::client::Client;
//! use google_cloud_spanner::mutation::insert;
//! use google_cloud_spanner::statement::Statement;
//! use google_cloud_spanner::reader::AsyncIterator;
//! use google_cloud_spanner::value::CommitTimestamp;
//! use google_cloud_spanner::client::RunInTxError;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!
//!     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
//!
//!     // Create spanner client
//!     let mut client = Client::new(DATABASE).await?;
//!
//!     // Insert
//!     let mutation = insert("Guild", &["GuildId", "OwnerUserID", "UpdatedAt"], &[&"guildId", &"ownerId", &CommitTimestamp::new()]);
//!     let commit_timestamp = client.apply(vec![mutation]).await?;
//!
//!     // Read with query
//!     let mut stmt = Statement::new("SELECT GuildId FROM Guild WHERE OwnerUserID = @OwnerUserID");
//!     stmt.add_param("OwnerUserID",&"ownerId");
//!     let mut tx = client.single().await?;
//!     let mut iter = tx.query(stmt).await?;
//!     while let Some(row) = iter.next(None).await? {
//!         let guild_id = row.column_by_name::<String>("GuildId");
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Example
//! Here is the example with using Warp.
//! * <https://github.com/yoshidan/google-cloud-rust-example/tree/main/spanner/rust>
//!
//! ## <a name="Documentation"></a>Documentation
//!
//! ### Overview
//! * [Creating a Client](#CreatingAClient)
//! * [Simple Reads and Writes](#SimpleReadsAndWrites)
//! * [Keys](#Keys)
//! * [KeyRanges](#KeyRanges)
//! * [KeySets](#KeySets)
//! * [Transactions](#Transactions)
//! * [Single Reads](#SingleReads)
//! * [Statements](#Statements)
//! * [Rows](#Rows)
//! * [Multiple Reads](#MultipleReads)
//! * [Timestamps and Timestamp Bounds](#TimestampsAndTimestampBounds)
//! * [Mutations](#Mutations)
//! * [Writes](#Writes)
//! * [Structs](#Structs)
//! * [DML and Partitioned DML](#DMLAndPartitionedDML)
//!
//! Package spanner provides a client for reading and writing to Cloud Spanner databases.
//! See the packages under admin for clients that operate on databases and instances.
//!
//! ### <a name="CreatingAClient"></a>Creating a Client
//!
//! To start working with this package, create a client that refers to the database of interest:
//!
//! ```
//! use google_cloud_spanner::client::Client;
//!
//! const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
//! let client = Client::new(DATABASE).await?;
//!
//! client.close().await;
//! ```
//!
//! Remember to close the client after use to free up the sessions in the session pool.
//!
//! To use an emulator with this library, you can set the SPANNER_EMULATOR_HOST environment variable to the address at which your emulator is running. This will send requests to that address instead of to Cloud Spanner.   You can then create and use a client as usual:
//!
//! ```ignore
//! use google_cloud_spanner::client::Client;
//!
//! // Set SPANNER_EMULATOR_HOST environment variable.
//! std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
//!
//! // Create client as usual.
//! const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
//! let mut client = match Client::new(DATABASE).await {
//!     Ok(client) => client,
//!     Err(e) => { /* handle error */ }
//! };
//! ```
//!
//! ### <a name="SimpleReadsAndWrites"></a>Simple Reads and Writes
//! Two Client methods, Apply and Single, work well for simple reads and writes. As a quick introduction, here we write a new row to the database and read it back:
//!
//! ```ignore
//! use google_cloud_spanner::mutation::insert;
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::value::CommitTimestamp;
//! use google_cloud_spanner::statement::ToKind;
//!
//! let mutation = insert("User",
//!     &["UserID", "Name", "UpdatedAt"], // columns
//!     &[&1, &"name", &CommitTimestamp::new()] // values
//! );
//! let commit_timestamp = client.apply(vec![mutation]).await?;
//!
//! let mut tx = client.single().await?;
//! let row = tx.read_row( "User", &["UserID", "Name", "UpdatedAt"], Key::key(&1)).await?;
//! ```
//!
//! All the methods used above are discussed in more detail below.
//!
//! ### <a name="Keys"></a>Keys
//!
//! Every Cloud Spanner row has a unique key, composed of one or more columns. Construct keys with a literal of type Key:
//!
//! ```
//! use google_cloud_spanner::key::Key;
//!
//! let key1 = Key::key(&"key");
//! ```
//!
//! ### <a name="KeyRanges"></a>KeyRanges
//!
//! The keys of a Cloud Spanner table are ordered. You can specify ranges of keys using the KeyRange type:
//!
//! ```
//! use google_cloud_spanner::key::{Key,KeyRange,RangeKind};
//!
//! let range1 = KeyRange::new(Key::key(&1), Key::key(&100), RangeKind::ClosedClosed);
//! let range2 = KeyRange::new(Key::key(&1), Key::key(&100), RangeKind::ClosedOpen);
//! let range3 = KeyRange::new(Key::key(&1), Key::key(&100), RangeKind::OpenOpen);
//! let range4 = KeyRange::new(Key::key(&1), Key::key(&100), RangeKind::OpenClosed);
//! ```
//!
//! ### <a name="KeySets"></a>KeySets
//!
//! A KeySet represents a set of keys. A single Key or KeyRange can act as a KeySet.
//!
//! ```
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::statement::ToKind;
//!
//! let key1 = Key::composite(&[&"Bob", &"2014-09-23"]);
//! let key2 = Key::composite(&[&"Alfred", &"2015-06-12"]);
//! let keys  = vec![key1,key2] ;
//! let composite_keys = vec![
//!     Key::composite(&[&"composite-pk-1-1",&"composite-pk-1-2"]),
//!     Key::composite(&[&"composite-pk-2-1",&"composite-pk-2-2"])
//! ];
//! ```
//!
//! all_keys returns a KeySet that refers to all the keys in a table:
//!
//! ```
//! use google_cloud_spanner::key::all_keys;
//!
//! let ks = all_keys();
//! ```
//!
//! ### <a name="Transactions"></a>Transactions
//!
//! All Cloud Spanner reads and writes occur inside transactions. There are two types of transactions, read-only and read-write. Read-only transactions cannot change the database, do not acquire locks, and may access either the current database state or states in the past. Read-write transactions can read the database before writing to it, and always apply to the most recent database state.
//!
//! ### <a name="SingleReads"></a>Single Reads
//! The simplest and fastest transaction is a ReadOnlyTransaction that supports a single read operation. Use Client.Single to create such a transaction. You can chain the call to Single with a call to a Read method.
//!
//! When you only want one row whose key you know, use ReadRow. Provide the table name, key, and the columns you want to read:
//!
//! ```ignore
//! use google_cloud_spanner::key::Key;
//!
//! let mut tx = client.single().await?;
//! let row = tx.read_row("Table", &["col1", "col2"], Key::key(&1)).await?;
//! ```
//!
//! Read multiple rows with the Read method. It takes a table name, KeySet, and list of columns:
//!
//! ```ignore
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::statement::ToKind;
//!
//! let mut tx = client.single().await?;
//! let iter1 = tx.read("Table",&["col1", "col2"], vec![
//!     Key::key(&"pk1"),
//!     Key::key(&"pk2")
//! ]).await?;
//! ```
//!
//! RowIterator also follows the standard pattern for the Google Cloud Client Libraries:
//!
//! ```ignore
//! use google_cloud_spanner::key::Key;
//!
//! let mut tx = client.single().await?;
//! let mut iter = tx.read("Table", &["col1", "col2"], vec![
//!     Key::key(&"pk1"),
//!     Key::key(&"pk2")
//! ]).await?;
//!
//! while let Some(row) = iter.next().await? {
//!     // use row
//! };
//! ```
//!
//! * The used session is returned to the drop timing session pool, so unlike Go, there is no need to call Stop.
//!
//! * To read rows with an index, use `client.read_with_option`.
//!
//! ### <a name="Statements"></a>Statements
//!
//! The most general form of reading uses SQL statements. Construct a Statement with NewStatement, setting any parameters using the Statement's Params map:
//!
//! ```
//! use google_cloud_spanner::statement::Statement;
//!
//! let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
//! stmt.add_param("UserId", &"user_id");
//! ```
//!
//! You can also construct a Statement directly with a struct literal, providing your own map of parameters.
//!
//! Use the Query method to run the statement and obtain an iterator:
//!
//! ```ignore
//! let mut tx = client.single().await?;
//! let iter = tx.query(stmt).await?;
//! ```
//!
//! ### <a name="Rows"></a>Rows
//! Once you have a Row, via an iterator or a call to read_row, you can extract column values in several ways. Pass in a pointer to a Rust variable of the appropriate type when you extract a value.
//!
//! You can extract by column position or name:
//!
//! ```ignore
//! let value           = row.column::<String>(0)?;
//! let nullable_value  = row.column::<Option<String>>(1)?;
//! let array_value     = row.column_by_name::<Vec<i64>>("array")?;
//! let struct_data     = row.column_by_name::<Vec<User>>("struct_data")?;
//! ```
//!
//! Or you can define a Rust struct that corresponds to your columns, and extract into that:
//! * `TryFromStruct` trait is required
//!
//! ```
//! use google_cloud_spanner::row::TryFromStruct;
//! use google_cloud_spanner::row::Struct;
//! use google_cloud_spanner::row::Error;
//!
//! pub struct User {
//!     pub user_id: String,
//!     pub premium: bool,
//!     pub updated_at: chrono::DateTime<chrono::Utc>
//! }
//!
//! impl TryFromStruct for User {
//!     fn try_from_struct(s: Struct<'_>) -> Result<Self, Error> {
//!         Ok(User {
//!             user_id: s.column_by_name("UserId")?,
//!             premium: s.column_by_name("Premium")?,
//!             updated_at: s.column_by_name("UpdatedAt")?,
//!         })
//!     }
//! }
//! ```
//!
//! ### <a name="MultipleReads"></a>Multiple Reads
//!
//! To perform more than one read in a transaction, use ReadOnlyTransaction:
//!
//! ```ignore
//! use google_cloud_spanner::statement::Statement;
//! use google_cloud_spanner::key::Key;
//!
//! let tx = client.read_only_transaction().await?;
//!
//! let mut stmt = Statement::new("SELECT * , \
//!             ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
//!             ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
//!             FROM User \
//!             WHERE UserId = @Param1");
//!
//! stmt.add_param("Param1", user_id);
//! let mut reader = tx.query(stmt).await?;
//! while let Some(row) = reader.next().await? {
//!     let user_id= row.column_by_name::<String>("UserId")?;
//!     let user_items= row.column_by_name::<Vec<model::UserItem>>("UserItem")?;
//!     let user_characters = row.column_by_name::<Vec<model::UserCharacter>>("UserCharacter")?;
//!     data.push(user_id);
//! }
//!
//! let mut reader2 = tx.read("User", &["UserId"], vec![
//!     Key::key(&"user-1"),
//!     Key::key(&"user-2")
//! ]).await?;
//!
//! // iterate reader2 ...
//!
//! let mut reader3 = tx.read("Table", &["col1", "col2"], vec![
//!     Key::composite(&[&"composite-pk-1-1",&"composite-pk-1-2"]),
//!     Key::composite(&[&"composite-pk-2-1",&"composite-pk-2-2"])
//! ]).await?;
//! // iterate reader3 ...
//! ```
//!
//! * The used session is returned to the drop timing session pool, so unlike Go, there is no need to call txn Close.
//!
//! ### <a name="TimestampsAndTimestampBounds"></a>Timestamps and Timestamp Bounds
//!
//! Cloud Spanner read-only transactions conceptually perform all their reads at a single moment in time, called the transaction's read timestamp. Once a read has started, you can call ReadOnlyTransaction's Timestamp method to obtain the read timestamp.
//!
//! By default, a transaction will pick the most recent time (a time where all previously committed transactions are visible) for its reads. This provides the freshest data, but may involve some delay. You can often get a quicker response if you are willing to tolerate "stale" data.
//! You can control the read timestamp selected by a transaction. For example, to perform a query on data that is at most one minute stale, use
//!
//! ```ignore
//! use google_cloud_spanner::value::TimestampBound;
//!
//! let tx = client.single_with_timestamp_bound(TimestampBound::max_staleness(std::time::Duration::from_secs(60))).await?;
//! ```
//!
//! See the documentation of TimestampBound for more details.
//!
//! ### <a name="Mutations"></a>Mutations
//!
//! To write values to a Cloud Spanner database, construct a Mutation. The spanner package has functions for inserting, updating and deleting rows. Except for the Delete methods, which take a Key or KeyRange, each mutation-building function comes in three varieties.
//!
//! One takes lists of columns and values along with the table name:
//!
//! ```
//! use google_cloud_spanner::mutation::insert;
//! use google_cloud_spanner::mutation::insert_map;
//! use google_cloud_spanner::value::CommitTimestamp;
//!
//! let mutation = insert("User",
//!     &[&"UserID", &"Name", &"UpdatedAt"], // columns
//!     &[&1, &"name", &CommitTimestamp::new()] // values
//! );
//! // or use insert_map
//! let mutation2 = insert_map("User",
//!     &[("UserID", &2), ("UserID", &"name2"), (&"UpdatedAt",&CommitTimestamp::new())]);
//! ```
//!
//! And the third accepts a struct value, and determines the columns from the struct field names:
//!
//! * `ToStruct` trait is required
//!
//! ```
//! use google_cloud_spanner::statement::Kinds;
//! use google_cloud_spanner::statement::Types;
//! use google_cloud_spanner::statement::ToStruct;
//! use google_cloud_spanner::statement::ToKind;
//! use google_cloud_spanner::value::CommitTimestamp;
//! use google_cloud_spanner::mutation::insert_or_update_struct;
//!
//! pub struct User {
//!     pub user_id: String,
//!     pub premium: bool,
//!     pub updated_at: chrono::DateTime<chrono::Utc>
//! }
//!
//! impl ToStruct for User {
//!     fn to_kinds(&self) -> Kinds {
//!         vec![
//!             ("UserId", self.user_id.to_kind()),
//!             ("Premium", self.premium.to_kind()),
//!             ("UpdatedAt", CommitTimestamp::new().to_kind())
//!         ]
//!     }
//!
//!     fn get_types() -> Types {
//!         vec![
//!             ("UserId", String::get_type()),
//!             ("Premium", bool::get_type()),
//!             ("UpdatedAt", CommitTimestamp::get_type())
//!         ]
//!     }
//! }
//!
//! let new_user = User {
//!     user_id: "user_id".to_string(),
//!     premium: true,
//!     updated_at: chrono::Utc::now(),
//! };
//! let m1 = insert_or_update_struct("User", &new_user);
//! ```
//!
//! ### <a name="Writes"></a>Writes
//!
//! To apply a list of mutations to the database, use Apply:
//! ```ignore
//! use google_cloud_spanner::mutation::insert;
//! use google_cloud_spanner::mutation::delete;
//! use google_cloud_spanner::key::all_keys;
//! use google_cloud_spanner::statement::ToKind;
//!
//! let m1 = delete("Table", all_keys());
//! let m2 = insert("Table", &["col1", "col2"], &[&"1", &"2"]);
//! let commit_timestamp = client.apply(vec![m1,m2]).await?;
//! ```
//!
//! If you need to read before writing in a single transaction, use a ReadWriteTransaction. ReadWriteTransactions may be aborted automatically by the backend and need to be retried. You pass in a function to ReadWriteTransaction, and the client will handle the retries automatically. Use the transaction's BufferWrite method to buffer mutations, which will all be executed at the end of the transaction:
//!
//! ```ignore
//! use google_cloud_spanner::mutation::update;
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::value::Timestamp;
//! use google_cloud_spanner::client::RunInTxError;
//!
//! let tx_result: Result<(Option<Timestamp>,()), RunInTxError> = client.read_write_transaction(|tx| {
//!     Box::pin(async move {
//!         // The transaction function will be called again if the error code
//!         // of this error is Aborted. The backend may automatically abort
//!         // any read/write transaction if it detects a deadlock or other problems.
//!         let mut reader = tx.read("UserItem", &["UserId", "ItemId", "Quantity"], Key::key(&"user1")).await?;
//!         let mut ms = vec![];
//!         while let Some(row) = reader.next().await? {
//!             let item_id = row.column_by_name::<i64>("ItemId")?;
//!             let quantity = row.column_by_name::<i64>("Quantity")? + 1;
//!             let m = update("UserItem", &["Quantity"], &[&user_id, &item_id, quantity + 1]);
//!             ms.push(m);
//!         }
//!         // The buffered mutation will be committed.  If the commit
//!         // fails with an Aborted error, this function will be called again
//!         tx.buffer_write(ms);
//!         Ok(())
//!     })
//! }).await;
//! ```
//!
//! You can customize error. The Error of the `read_write_transaction` must implements
//! * `From<google_cloud_googleapis::Status>`
//! * `From<google_cloud_spanner::session::SessionError>`
//! * `google_cloud_gax::invoke::TryAs<google_cloud_googleapis::Status>`
//!
//! ### <a name="DMLAndPartitionedDML"></a>DML and Partitioned DML
//! For large databases, it may be more efficient to partition the DML statement.
//! Use client.partitioned_update to run a DML statement in this way. Not all DML statements can be partitioned.
//!
//! ```ignore
//! use google_cloud_spanner::client::Client;
//! use google_cloud_spanner::statement::Statement;
//!
//! let client = Client::new(DATABASE).await?;
//! let stmt = Statement::new("UPDATE User SET Value = 'aaa' WHERE Value IS NOT NULL");
//! let result = client.partitioned_update(stmt).await?;
//! ```
pub const AUDIENCE: &str = "https://spanner.googleapis.com/";
pub const SPANNER: &str = "spanner.googleapis.com";

pub mod admin;
pub mod apiv1;
pub mod client;
pub mod key;
pub mod mutation;
pub mod reader;
pub mod retry;
pub mod row;
pub mod session;
pub mod statement;
pub mod transaction;
pub mod transaction_ro;
pub mod transaction_rw;
pub mod value;

#![allow(clippy::result_large_err)]
//! # google-cloud-spanner
//!
//! Google Cloud Platform spanner library.
//!
//! * [About Cloud Spanner](https://cloud.google.com/spanner/)
//! * [Spanner API Documentation](https://cloud.google.com/spanner/docs)
//! * [Rust client Documentation](#Documentation)
//!
//! ## Quickstart
//! Create `Client` and call transaction API same as [Google Cloud Go](https://github.com/googleapis/google-cloud-go/tree/main/spanner).
//!
//! ```
//! use google_cloud_spanner::client::Client;
//! use google_cloud_spanner::mutation::insert_or_update;
//! use google_cloud_spanner::statement::Statement;
//! use google_cloud_spanner::value::CommitTimestamp;
//! use google_cloud_spanner::client::Error;
//! use google_cloud_spanner::client::ClientConfig;
//! use google_cloud_gax::grpc::Status;
//!
//! async fn run(config: ClientConfig) -> Result<(), Error>{
//!
//!     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
//!
//!     // Create spanner client
//!     let mut client = Client::new(DATABASE, config).await?;
//!
//!     // Insert or update
//!     let mutation = insert_or_update("Guild", &["GuildId", "OwnerUserID", "UpdatedAt"], &[&"guildId", &"ownerId", &CommitTimestamp::new()]);
//!     let commit_timestamp = client.apply(vec![mutation]).await?;
//!
//!     // Read with query
//!     let mut stmt = Statement::new("SELECT GuildId FROM Guild WHERE OwnerUserID = @OwnerUserID");
//!     stmt.add_param("OwnerUserID",&"ownerId");
//!     let mut tx = client.single().await?;
//!     let mut iter = tx.query(stmt).await?;
//!     while let Some(row) = iter.next().await? {
//!         let guild_id = row.column_by_name::<String>("GuildId");
//!         // do something
//!     }
//!
//!     // Remove all the sessions.
//!     client.close().await;
//!     Ok(())
//! }
//! ```
//!
//! ## Related project
//! * [google-cloud-spanner-derive](https://github.com/yoshidan/google-cloud-rust/spanner-derive)
//!
//! ## <a name="Documentation"></a>Documentation
//!
//! ### Overview
//! * [Creating a Client](#CreatingAClient)
//! * [Authentication](#Authentication)
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
//! use google_cloud_spanner::client::ClientConfig;
//!
//! async fn run() {
//!     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
//!
//!     // google_cloud_default provides default ClientConfig with credentials source
//!     let config = ClientConfig::default().with_auth().await.unwrap();
//!     let mut client = Client::new(DATABASE, config).await.unwrap();
//!
//!     client.close().await;
//! }
//! ```
//!
//! Remember to close the client after use to free up the sessions in the session pool.
//!
//! To use an emulator with this library, you can set the SPANNER_EMULATOR_HOST environment variable to the address at which your emulator is running. This will send requests to that address instead of to Cloud Spanner.   You can then create and use a client as usual:
//!
//! ```
//! use google_cloud_spanner::client::Client;
//! use google_cloud_spanner::client::ClientConfig;
//! use google_cloud_spanner::client::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error>{
//!     // Set SPANNER_EMULATOR_HOST environment variable.
//!     std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
//!
//!     // Create client as usual.
//!     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
//!     let client = Client::new(DATABASE, ClientConfig::default()).await?;
//!     Ok(())
//! }
//! ```
//!
//! ### <a name="Authentication"></a>Authentication
//!
//! There are two ways to create a client that is authenticated against the google cloud.
//!
//! #### Automatically
//!
//! The function `with_auth()` will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
//! from a metadata server.
//!
//! This is also described in [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust/blob/main/foundation/auth/README.md)
//!
//! ```
//! use google_cloud_spanner::client::{ClientConfig, Client};
//!
//! async fn run() {
//!     let config = ClientConfig::default().with_auth().await.unwrap();
//!     let client = Client::new("projects/project/instances/instance/databases/database",config).await.unwrap();
//! }
//! ```
//!
//! ### Manually
//!
//! When you can't use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
//! you can parse your own version of the 'credentials-file' and use it like that:
//!
//! ```
//! use google_cloud_auth::credentials::CredentialsFile;
//! // or google_cloud_spanner::client::google_cloud_auth::credentials::CredentialsFile
//! use google_cloud_spanner::client::{ClientConfig, Client};
//!
//! async fn run(cred: CredentialsFile) {
//!     let config = ClientConfig::default().with_credentials(cred).await.unwrap();
//!     let client = Client::new("projects/project/instances/instance/databases/database",config).await.unwrap();
//! }
//! ```
//!
//! ### <a name="SimpleReadsAndWrites"></a>Simple Reads and Writes
//! Two Client methods, Apply and Single, work well for simple reads and writes. As a quick introduction, here we write a new row to the database and read it back:
//!
//! ```
//! use google_cloud_spanner::mutation::insert;
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::value::CommitTimestamp;
//! use google_cloud_spanner::statement::ToKind;
//! use google_cloud_spanner::client::{Client, Error};
//! use google_cloud_spanner::mutation::insert_or_update;
//!
//! async fn run(client: Client) -> Result<(), Error>{
//!     let mutation = insert_or_update("Guild", &["GuildId", "OwnerUserID", "UpdatedAt"], &[&"guildId1", &"ownerId1", &CommitTimestamp::new()]);
//!     let commit_timestamp = client.apply(vec![mutation]).await?;
//!
//!     let mut tx = client.single().await?;
//!     let row = tx.read_row( "Guild", &["GuildId", "OwnerUserID", "UpdatedAt"], Key::new(&"guildId1")).await?;
//!     Ok(())
//! }
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
//! let key1 = Key::new(&"key");
//! ```
//!
//! ### <a name="KeyRanges"></a>KeyRanges
//!
//! The keys of a Cloud Spanner table are ordered. You can specify ranges of keys using the KeyRange type:
//!
//! ```
//! use google_cloud_spanner::key::{Key,KeyRange,RangeKind};
//!
//! let range1 = KeyRange::new(Key::new(&1), Key::new(&100), RangeKind::ClosedClosed);
//! let range2 = KeyRange::new(Key::new(&1), Key::new(&100), RangeKind::ClosedOpen);
//! let range3 = KeyRange::new(Key::new(&1), Key::new(&100), RangeKind::OpenOpen);
//! let range4 = KeyRange::new(Key::new(&1), Key::new(&100), RangeKind::OpenClosed);
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
//! ```
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::client::{Client, Error};
//!
//! async fn run(client: Client) -> Result<(), Error>{
//!     let mut tx = client.single().await?;
//!     let row = tx.read_row("Guild", &["GuildID", "OwnerUserID"], Key::new(&"guild1")).await;
//!     Ok(())
//! }
//! ```
//!
//! Read multiple rows with the Read method. It takes a table name, KeySet, and list of columns:
//!
//! ```
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::statement::ToKind;
//! use google_cloud_spanner::client::Client;
//! use google_cloud_spanner::client::Error;
//!
//! async fn run(client: Client) -> Result<(), Error>{
//!     let mut tx = client.single().await?;
//!     let iter1 = tx.read("Guild",&["GuildID", "OwnerUserID"], vec![
//!         Key::new(&"pk1"),
//!         Key::new(&"pk2")
//!     ]).await?;
//!     Ok(())
//! }
//! ```
//!
//! RowIterator also follows the standard pattern for the Google Cloud Client Libraries:
//!
//! ```
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::client::Client;
//! use google_cloud_spanner::client::Error;
//!
//! #[tokio::main]
//! async fn run(client: Client) -> Result<(), Error>{
//!     let mut tx = client.single().await?;
//!     let mut iter = tx.read("Guild", &["GuildID", "OwnerUserID"], vec![
//!         Key::new(&"pk1"),
//!         Key::new(&"pk2")
//!     ]).await.unwrap();
//!
//!     while let Some(row) = iter.next().await? {
//!         let guild_id = row.column_by_name::<String>("GuildID");
//!         //do something
//!     };
//!     Ok(())
//! }
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
//! ```
//! use google_cloud_spanner::client::{Client, Error};
//! use google_cloud_spanner::statement::Statement;
//!
//! async fn run(client: Client) -> Result<(), Error>{
//!     let mut stmt = Statement::new("SELECT * FROM Guild WHERE OwnerUserID = @OwnerUserID");
//!     stmt.add_param("OwnerUserID", &"key");
//!     let mut tx = client.single().await?;
//!     let iter = tx.query(stmt).await?;
//!     Ok(())
//! }
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
//!     pub updated_at: time::OffsetDateTime,
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
//! use google_cloud_spanner::client::{Client, Error};
//! use google_cloud_spanner::statement::Statement;
//! use google_cloud_spanner::key::Key;
//!
//! async fn run(client: Client) -> Result<(), Error> {
//!     let mut tx = client.read_only_transaction().await?;
//!
//!     let mut stmt = Statement::new("SELECT * , \
//!             ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
//!             ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
//!             FROM User \
//!             WHERE UserId = @Param1");
//!
//!     stmt.add_param("Param1", user_id);
//!     let mut reader = tx.query(stmt).await?;
//!     let mut data = vec![];
//!     while let Some(row) = reader.next().await? {
//!         let user_id= row.column_by_name::<String>("UserId")?;
//!         let user_items= row.column_by_name::<Vec<model::UserItem>>("UserItem")?;
//!         let user_characters = row.column_by_name::<Vec<model::UserCharacter>>("UserCharacter")?;
//!         data.push(user_id);
//!     }
//!
//!     let mut reader2 = tx.read("User", &["UserId"], vec![
//!         Key::new(&"user-1"),
//!         Key::new(&"user-2")
//!     ]).await?;
//!
//!     // iterate reader2 ...
//!
//!     let mut reader3 = tx.read("Table", &["col1", "col2"], vec![
//!         Key::composite(&[&"composite-pk-1-1",&"composite-pk-1-2"]),
//!         Key::composite(&[&"composite-pk-2-1",&"composite-pk-2-2"])
//!     ]).await?;
//!
//!     Ok(())
//! }
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
//! ```
//! use google_cloud_spanner::client::{Client, Error};
//! use google_cloud_spanner::value::TimestampBound;
//!
//! pub async fn run(client: Client) -> Result<(), Error>{
//!     let tx = client.single_with_timestamp_bound(TimestampBound::max_staleness(std::time::Duration::from_secs(60))).await?;
//!     Ok(())
//! }
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
//! use google_cloud_spanner::mutation::insert_or_update;
//! use google_cloud_spanner::mutation::insert_or_update_map;
//! use google_cloud_spanner::value::CommitTimestamp;
//! use google_cloud_spanner::client::Client;
//!
//! fn run(client: Client) {
//!     let mutation = insert_or_update("Guild",
//!         &[&"GuildID", &"OwnerUserID", &"UpdatedAt"], // columns
//!         &[&"gid", &"owner", &CommitTimestamp::new()] // values
//!     );
//!     // or use insert_map
//!     let mutation2 = insert_or_update_map("Guild",
//!         &[("GuildId", &"gid"), ("OwnerUserID", &"owner"), (&"UpdatedAt",&CommitTimestamp::new())]
//!     );
//! }
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
//!     pub updated_at: time::OffsetDateTime,
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
//!     updated_at: time::OffsetDateTime::now_utc(),
//! };
//! let m1 = insert_or_update_struct("User", &new_user);
//! ```
//!
//! ### <a name="Writes"></a>Writes
//!
//! To apply a list of mutations to the database, use Apply:
//! ```
//! use google_cloud_spanner::mutation::insert;
//! use google_cloud_spanner::mutation::delete;
//! use google_cloud_spanner::key::all_keys;
//! use google_cloud_spanner::statement::ToKind;
//! use google_cloud_spanner::value::CommitTimestamp;
//! use google_cloud_spanner::client::{Client, Error};
//!
//! async fn run(client: Client) -> Result<(), Error>{
//!     let m1 = delete("Guild", all_keys());
//!     let m2 = insert("Guild", &["GuildID", "OwnerUserID", "UpdatedAt"], &[&"1", &"2", &CommitTimestamp::new()]);
//!     let commit_timestamp = client.apply(vec![m1,m2]).await?;
//!     Ok(())
//! }
//! ```
//!
//! If you need to read before writing in a single transaction, use a ReadWriteTransaction. ReadWriteTransactions may be aborted automatically by the backend and need to be retried. You pass in a function to ReadWriteTransaction, and the client will handle the retries automatically. Use the transaction's BufferWrite method to buffer mutations, which will all be executed at the end of the transaction:
//!
//! ```
//! use google_cloud_spanner::mutation::update;
//! use google_cloud_spanner::key::Key;
//! use google_cloud_spanner::value::Timestamp;
//! use google_cloud_spanner::client::Error;
//! use google_cloud_spanner::client::Client;
//!
//! async fn run(client: Client) ->Result<(Option<Timestamp>,()), Error> {
//!     client.read_write_transaction(|tx| {
//!         Box::pin(async move {
//!             // The transaction function will be called again if the error code
//!             // of this error is Aborted. The backend may automatically abort
//!             // any read/write transaction if it detects a deadlock or other problems.
//!             let key = Key::new(&"user1");
//!             let mut reader = tx.read("UserItem", &["UserId", "ItemId", "Quantity"], key).await?;
//!             let mut ms = vec![];
//!             while let Some(row) = reader.next().await? {
//!                 let user_id = row.column_by_name::<i64>("UserId")?;
//!                 let item_id = row.column_by_name::<i64>("ItemId")?;
//!                 let quantity = row.column_by_name::<i64>("Quantity")? + 1;
//!                 let m = update("UserItem", &["UserId", "ItemId", "Quantity"], &[&user_id, &item_id, &quantity]);
//!                 ms.push(m);
//!             }
//!             // The buffered mutation will be committed.  If the commit
//!             // fails with an Aborted error, this function will be called again
//!             tx.buffer_write(ms);
//!             Ok(())
//!         })
//!     }).await
//! }
//! ```
//!
//! You can customize error. The Error of the `read_write_transaction` must implements
//! * `From<google_cloud_googleapis::Status>`
//! * `From<google_cloud_spanner::session::SessionError>`
//! * `google_cloud_gax::invoke::TryAs<google_cloud_googleapis::Status>`
//! ```
//! use google_cloud_gax::grpc::Status;
//! use google_cloud_gax::retry::TryAs;
//! use google_cloud_spanner::client::Error;
//! use google_cloud_spanner::session::SessionError;
//!
//! #[derive(thiserror::Error, Debug)]
//! pub enum DomainError {
//!     #[error("invalid")]
//!     OtherError,
//!     #[error(transparent)]
//!     Tx(#[from] Error),
//! }
//!
//! impl TryAs<Status> for DomainError {
//! fn try_as(&self) -> Option<&Status> {
//!     match self {
//!         DomainError::Tx(Error::GRPC(status)) => Some(status),
//!         _ => None,
//!     }
//!  }
//! }
//! impl From<Status> for DomainError {
//!     fn from(status: Status) -> Self {
//!         Self::Tx(Error::GRPC(status))
//!     }
//! }
//! impl From<SessionError> for DomainError {
//!     fn from(se: SessionError) -> Self {
//!         Self::Tx(Error::InvalidSession(se))
//!     }
//!  }
//! ```
//!
//! You can begin transaction  by `begin_read_write_transaction`.
//! It is necessary to write retry processing for transaction abort
//! ```
//! use google_cloud_spanner::mutation::update;
//! use google_cloud_spanner::key::{Key, all_keys};
//! use google_cloud_spanner::value::Timestamp;
//! use google_cloud_spanner::client::Error;
//! use google_cloud_spanner::client::Client;
//! use google_cloud_spanner::transaction_rw::ReadWriteTransaction;
//! use google_cloud_googleapis::spanner::v1::execute_batch_dml_request::Statement;
//! use google_cloud_spanner::retry::TransactionRetry;
//!
//! async fn run(client: Client) -> Result<(), Error> {
//!     let retry = &mut TransactionRetry::new();
//!     loop {
//!         let tx = &mut client.begin_read_write_transaction().await?;
//!
//!         let result = run_in_transaction(tx).await;
//!
//!         // try to commit or rollback transaction.
//!         match tx.end(result, None).await {
//!             Ok((_commit_timestamp, success)) => return Ok(success),
//!             Err(err) => retry.next(err).await? // check retry
//!         }
//!     }
//! }
//!
//! async fn run_in_transaction(tx: &mut ReadWriteTransaction) -> Result<(), Error> {
//!     let key = all_keys();
//!     let mut reader = tx.read("UserItem", &["UserId", "ItemId", "Quantity"], key).await?;
//!     let mut ms = vec![];
//!     while let Some(row) = reader.next().await? {
//!         let user_id = row.column_by_name::<String>("UserId")?;
//!         let item_id = row.column_by_name::<i64>("ItemId")?;
//!         let quantity = row.column_by_name::<i64>("Quantity")? + 1;
//!         let m = update("UserItem", &["UserId", "ItemId", "Quantity"], &[&user_id, &item_id, &quantity]);
//!         ms.push(m);
//!     }
//!     tx.buffer_write(ms);
//!     Ok(())
//! }
//! ```
//!
//! ### <a name="DMLAndPartitionedDML"></a>DML and Partitioned DML
//! For large databases, it may be more efficient to partition the DML statement.
//! Use client.partitioned_update to run a DML statement in this way. Not all DML statements can be partitioned.
//!
//! ```
//! use google_cloud_spanner::client::{Client, Error};
//! use google_cloud_spanner::statement::Statement;
//!
//! #[tokio::main]
//! async fn run(client:Client) -> Result<(), Error>{
//!     let stmt = Statement::new("UPDATE User SET NullableString = 'aaa' WHERE NullableString IS NOT NULL");
//!     let result = client.partitioned_update(stmt).await?;
//!     Ok(())
//! }
//! ```
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
pub use bigdecimal;

/// The response for \[Commit][google.spanner.v1.Spanner.Commit\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitResponse {
    /// The Cloud Spanner timestamp at which the transaction committed.
    #[prost(message, optional, tag = "1")]
    pub commit_timestamp: ::core::option::Option<::prost_types::Timestamp>,
    /// The statistics about this Commit. Not returned by default.
    /// For more information, see
    /// \[CommitRequest.return_commit_stats][google.spanner.v1.CommitRequest.return_commit_stats\].
    #[prost(message, optional, tag = "2")]
    pub commit_stats: ::core::option::Option<commit_response::CommitStats>,
}
/// Nested message and enum types in `CommitResponse`.
pub mod commit_response {
    /// Additional statistics about a commit.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CommitStats {
        /// The total number of mutations for the transaction. Knowing the
        /// `mutation_count` value can help you maximize the number of mutations
        /// in a transaction and minimize the number of API round trips. You can
        /// also monitor this value to prevent transactions from exceeding the system
        /// \[limit\](<https://cloud.google.com/spanner/quotas#limits_for_creating_reading_updating_and_deleting_data>).
        /// If the number of mutations exceeds the limit, the server returns
        /// \[INVALID_ARGUMENT\](<https://cloud.google.com/spanner/docs/reference/rest/v1/Code#ENUM_VALUES.INVALID_ARGUMENT>).
        #[prost(int64, tag = "1")]
        pub mutation_count: i64,
    }
}
/// KeyRange represents a range of rows in a table or index.
///
/// A range has a start key and an end key. These keys can be open or
/// closed, indicating if the range includes rows with that key.
///
/// Keys are represented by lists, where the ith value in the list
/// corresponds to the ith component of the table or index primary key.
/// Individual values are encoded as described
/// \[here][google.spanner.v1.TypeCode\].
///
/// For example, consider the following table definition:
///
///      CREATE TABLE UserEvents (
///        UserName STRING(MAX),
///        EventDate STRING(10)
///      ) PRIMARY KEY(UserName, EventDate);
///
/// The following keys name rows in this table:
///
///      ["Bob", "2014-09-23"]
///      ["Alfred", "2015-06-12"]
///
/// Since the `UserEvents` table's `PRIMARY KEY` clause names two
/// columns, each `UserEvents` key has two elements; the first is the
/// `UserName`, and the second is the `EventDate`.
///
/// Key ranges with multiple components are interpreted
/// lexicographically by component using the table or index key's declared
/// sort order. For example, the following range returns all events for
/// user `"Bob"` that occurred in the year 2015:
///
///      "start_closed": ["Bob", "2015-01-01"]
///      "end_closed": ["Bob", "2015-12-31"]
///
/// Start and end keys can omit trailing key components. This affects the
/// inclusion and exclusion of rows that exactly match the provided key
/// components: if the key is closed, then rows that exactly match the
/// provided components are included; if the key is open, then rows
/// that exactly match are not included.
///
/// For example, the following range includes all events for `"Bob"` that
/// occurred during and after the year 2000:
///
///      "start_closed": ["Bob", "2000-01-01"]
///      "end_closed": \["Bob"\]
///
/// The next example retrieves all events for `"Bob"`:
///
///      "start_closed": \["Bob"\]
///      "end_closed": \["Bob"\]
///
/// To retrieve events before the year 2000:
///
///      "start_closed": \["Bob"\]
///      "end_open": ["Bob", "2000-01-01"]
///
/// The following range includes all rows in the table:
///
///      "start_closed": []
///      "end_closed": []
///
/// This range returns all users whose `UserName` begins with any
/// character from A to C:
///
///      "start_closed": \["A"\]
///      "end_open": \["D"\]
///
/// This range returns all users whose `UserName` begins with B:
///
///      "start_closed": \["B"\]
///      "end_open": \["C"\]
///
/// Key ranges honor column sort order. For example, suppose a table is
/// defined as follows:
///
///      CREATE TABLE DescendingSortedTable {
///        Key INT64,
///        ...
///      ) PRIMARY KEY(Key DESC);
///
/// The following range retrieves all rows with key values between 1
/// and 100 inclusive:
///
///      "start_closed": \["100"\]
///      "end_closed": \["1"\]
///
/// Note that 100 is passed as the start, and 1 is passed as the end,
/// because `Key` is a descending column in the schema.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyRange {
    /// The start key must be provided. It can be either closed or open.
    #[prost(oneof = "key_range::StartKeyType", tags = "1, 2")]
    pub start_key_type: ::core::option::Option<key_range::StartKeyType>,
    /// The end key must be provided. It can be either closed or open.
    #[prost(oneof = "key_range::EndKeyType", tags = "3, 4")]
    pub end_key_type: ::core::option::Option<key_range::EndKeyType>,
}
/// Nested message and enum types in `KeyRange`.
pub mod key_range {
    /// The start key must be provided. It can be either closed or open.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum StartKeyType {
        /// If the start is closed, then the range includes all rows whose
        /// first `len(start_closed)` key columns exactly match `start_closed`.
        #[prost(message, tag = "1")]
        StartClosed(::prost_types::ListValue),
        /// If the start is open, then the range excludes rows whose first
        /// `len(start_open)` key columns exactly match `start_open`.
        #[prost(message, tag = "2")]
        StartOpen(::prost_types::ListValue),
    }
    /// The end key must be provided. It can be either closed or open.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum EndKeyType {
        /// If the end is closed, then the range includes all rows whose
        /// first `len(end_closed)` key columns exactly match `end_closed`.
        #[prost(message, tag = "3")]
        EndClosed(::prost_types::ListValue),
        /// If the end is open, then the range excludes rows whose first
        /// `len(end_open)` key columns exactly match `end_open`.
        #[prost(message, tag = "4")]
        EndOpen(::prost_types::ListValue),
    }
}
/// `KeySet` defines a collection of Cloud Spanner keys and/or key ranges. All
/// the keys are expected to be in the same table or index. The keys need
/// not be sorted in any particular way.
///
/// If the same key is specified multiple times in the set (for example
/// if two ranges, two keys, or a key and a range overlap), Cloud Spanner
/// behaves as if the key were only specified once.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeySet {
    /// A list of specific keys. Entries in `keys` should have exactly as
    /// many elements as there are columns in the primary or index key
    /// with which this `KeySet` is used.  Individual key values are
    /// encoded as described \[here][google.spanner.v1.TypeCode\].
    #[prost(message, repeated, tag = "1")]
    pub keys: ::prost::alloc::vec::Vec<::prost_types::ListValue>,
    /// A list of key ranges. See \[KeyRange][google.spanner.v1.KeyRange\] for more information about
    /// key range specifications.
    #[prost(message, repeated, tag = "2")]
    pub ranges: ::prost::alloc::vec::Vec<KeyRange>,
    /// For convenience `all` can be set to `true` to indicate that this
    /// `KeySet` matches all keys in the table or index. Note that any keys
    /// specified in `keys` or `ranges` are only yielded once.
    #[prost(bool, tag = "3")]
    pub all: bool,
}
/// A modification to one or more Cloud Spanner rows.  Mutations can be
/// applied to a Cloud Spanner database by sending them in a
/// \[Commit][google.spanner.v1.Spanner.Commit\] call.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Mutation {
    /// Required. The operation to perform.
    #[prost(oneof = "mutation::Operation", tags = "1, 2, 3, 4, 5")]
    pub operation: ::core::option::Option<mutation::Operation>,
}
/// Nested message and enum types in `Mutation`.
pub mod mutation {
    /// Arguments to \[insert][google.spanner.v1.Mutation.insert\], \[update][google.spanner.v1.Mutation.update\], \[insert_or_update][google.spanner.v1.Mutation.insert_or_update\], and
    /// \[replace][google.spanner.v1.Mutation.replace\] operations.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Write {
        /// Required. The table whose rows will be written.
        #[prost(string, tag = "1")]
        pub table: ::prost::alloc::string::String,
        /// The names of the columns in \[table][google.spanner.v1.Mutation.Write.table\] to be written.
        ///
        /// The list of columns must contain enough columns to allow
        /// Cloud Spanner to derive values for all primary key columns in the
        /// row(s) to be modified.
        #[prost(string, repeated, tag = "2")]
        pub columns: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// The values to be written. `values` can contain more than one
        /// list of values. If it does, then multiple rows are written, one
        /// for each entry in `values`. Each list in `values` must have
        /// exactly as many entries as there are entries in \[columns][google.spanner.v1.Mutation.Write.columns\]
        /// above. Sending multiple lists is equivalent to sending multiple
        /// `Mutation`s, each containing one `values` entry and repeating
        /// \[table][google.spanner.v1.Mutation.Write.table\] and \[columns][google.spanner.v1.Mutation.Write.columns\]. Individual values in each list are
        /// encoded as described \[here][google.spanner.v1.TypeCode\].
        #[prost(message, repeated, tag = "3")]
        pub values: ::prost::alloc::vec::Vec<::prost_types::ListValue>,
    }
    /// Arguments to \[delete][google.spanner.v1.Mutation.delete\] operations.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Delete {
        /// Required. The table whose rows will be deleted.
        #[prost(string, tag = "1")]
        pub table: ::prost::alloc::string::String,
        /// Required. The primary keys of the rows within \[table][google.spanner.v1.Mutation.Delete.table\] to delete.  The
        /// primary keys must be specified in the order in which they appear in the
        /// `PRIMARY KEY()` clause of the table's equivalent DDL statement (the DDL
        /// statement used to create the table).
        /// Delete is idempotent. The transaction will succeed even if some or all
        /// rows do not exist.
        #[prost(message, optional, tag = "2")]
        pub key_set: ::core::option::Option<super::KeySet>,
    }
    /// Required. The operation to perform.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Operation {
        /// Insert new rows in a table. If any of the rows already exist,
        /// the write or transaction fails with error `ALREADY_EXISTS`.
        #[prost(message, tag = "1")]
        Insert(Write),
        /// Update existing rows in a table. If any of the rows does not
        /// already exist, the transaction fails with error `NOT_FOUND`.
        #[prost(message, tag = "2")]
        Update(Write),
        /// Like \[insert][google.spanner.v1.Mutation.insert\], except that if the row already exists, then
        /// its column values are overwritten with the ones provided. Any
        /// column values not explicitly written are preserved.
        ///
        /// When using \[insert_or_update][google.spanner.v1.Mutation.insert_or_update\], just as when using \[insert][google.spanner.v1.Mutation.insert\], all `NOT
        /// NULL` columns in the table must be given a value. This holds true
        /// even when the row already exists and will therefore actually be updated.
        #[prost(message, tag = "3")]
        InsertOrUpdate(Write),
        /// Like \[insert][google.spanner.v1.Mutation.insert\], except that if the row already exists, it is
        /// deleted, and the column values provided are inserted
        /// instead. Unlike \[insert_or_update][google.spanner.v1.Mutation.insert_or_update\], this means any values not
        /// explicitly written become `NULL`.
        ///
        /// In an interleaved table, if you create the child table with the
        /// `ON DELETE CASCADE` annotation, then replacing a parent row
        /// also deletes the child rows. Otherwise, you must delete the
        /// child rows before you replace the parent row.
        #[prost(message, tag = "4")]
        Replace(Write),
        /// Delete rows from a table. Succeeds whether or not the named
        /// rows were present.
        #[prost(message, tag = "5")]
        Delete(Delete),
    }
}
/// Node information for nodes appearing in a \[QueryPlan.plan_nodes][google.spanner.v1.QueryPlan.plan_nodes\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlanNode {
    /// The `PlanNode`'s index in [node list]\[google.spanner.v1.QueryPlan.plan_nodes\].
    #[prost(int32, tag = "1")]
    pub index: i32,
    /// Used to determine the type of node. May be needed for visualizing
    /// different kinds of nodes differently. For example, If the node is a
    /// \[SCALAR][google.spanner.v1.PlanNode.Kind.SCALAR\] node, it will have a condensed representation
    /// which can be used to directly embed a description of the node in its
    /// parent.
    #[prost(enumeration = "plan_node::Kind", tag = "2")]
    pub kind: i32,
    /// The display name for the node.
    #[prost(string, tag = "3")]
    pub display_name: ::prost::alloc::string::String,
    /// List of child node `index`es and their relationship to this parent.
    #[prost(message, repeated, tag = "4")]
    pub child_links: ::prost::alloc::vec::Vec<plan_node::ChildLink>,
    /// Condensed representation for \[SCALAR][google.spanner.v1.PlanNode.Kind.SCALAR\] nodes.
    #[prost(message, optional, tag = "5")]
    pub short_representation: ::core::option::Option<plan_node::ShortRepresentation>,
    /// Attributes relevant to the node contained in a group of key-value pairs.
    /// For example, a Parameter Reference node could have the following
    /// information in its metadata:
    ///
    ///      {
    ///        "parameter_reference": "param1",
    ///        "parameter_type": "array"
    ///      }
    #[prost(message, optional, tag = "6")]
    pub metadata: ::core::option::Option<::prost_types::Struct>,
    /// The execution statistics associated with the node, contained in a group of
    /// key-value pairs. Only present if the plan was returned as a result of a
    /// profile query. For example, number of executions, number of rows/time per
    /// execution etc.
    #[prost(message, optional, tag = "7")]
    pub execution_stats: ::core::option::Option<::prost_types::Struct>,
}
/// Nested message and enum types in `PlanNode`.
pub mod plan_node {
    /// Metadata associated with a parent-child relationship appearing in a
    /// \[PlanNode][google.spanner.v1.PlanNode\].
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ChildLink {
        /// The node to which the link points.
        #[prost(int32, tag = "1")]
        pub child_index: i32,
        /// The type of the link. For example, in Hash Joins this could be used to
        /// distinguish between the build child and the probe child, or in the case
        /// of the child being an output variable, to represent the tag associated
        /// with the output variable.
        #[prost(string, tag = "2")]
        pub r#type: ::prost::alloc::string::String,
        /// Only present if the child node is \[SCALAR][google.spanner.v1.PlanNode.Kind.SCALAR\] and corresponds
        /// to an output variable of the parent node. The field carries the name of
        /// the output variable.
        /// For example, a `TableScan` operator that reads rows from a table will
        /// have child links to the `SCALAR` nodes representing the output variables
        /// created for each column that is read by the operator. The corresponding
        /// `variable` fields will be set to the variable names assigned to the
        /// columns.
        #[prost(string, tag = "3")]
        pub variable: ::prost::alloc::string::String,
    }
    /// Condensed representation of a node and its subtree. Only present for
    /// `SCALAR` \[PlanNode(s)][google.spanner.v1.PlanNode\].
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ShortRepresentation {
        /// A string representation of the expression subtree rooted at this node.
        #[prost(string, tag = "1")]
        pub description: ::prost::alloc::string::String,
        /// A mapping of (subquery variable name) -> (subquery node id) for cases
        /// where the `description` string of this node references a `SCALAR`
        /// subquery contained in the expression subtree rooted at this node. The
        /// referenced `SCALAR` subquery may not necessarily be a direct child of
        /// this node.
        #[prost(map = "string, int32", tag = "2")]
        pub subqueries: ::std::collections::HashMap<::prost::alloc::string::String, i32>,
    }
    /// The kind of \[PlanNode][google.spanner.v1.PlanNode\]. Distinguishes between the two different kinds of
    /// nodes that can appear in a query plan.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Kind {
        /// Not specified.
        Unspecified = 0,
        /// Denotes a Relational operator node in the expression tree. Relational
        /// operators represent iterative processing of rows during query execution.
        /// For example, a `TableScan` operation that reads rows from a table.
        Relational = 1,
        /// Denotes a Scalar node in the expression tree. Scalar nodes represent
        /// non-iterable entities in the query plan. For example, constants or
        /// arithmetic operators appearing inside predicate expressions or references
        /// to column names.
        Scalar = 2,
    }
    impl Kind {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Kind::Unspecified => "KIND_UNSPECIFIED",
                Kind::Relational => "RELATIONAL",
                Kind::Scalar => "SCALAR",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "KIND_UNSPECIFIED" => Some(Self::Unspecified),
                "RELATIONAL" => Some(Self::Relational),
                "SCALAR" => Some(Self::Scalar),
                _ => None,
            }
        }
    }
}
/// Contains an ordered list of nodes appearing in the query plan.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPlan {
    /// The nodes in the query plan. Plan nodes are returned in pre-order starting
    /// with the plan root. Each \[PlanNode][google.spanner.v1.PlanNode\]'s `id` corresponds to its index in
    /// `plan_nodes`.
    #[prost(message, repeated, tag = "1")]
    pub plan_nodes: ::prost::alloc::vec::Vec<PlanNode>,
}
/// Transactions:
///
/// Each session can have at most one active transaction at a time (note that
/// standalone reads and queries use a transaction internally and do count
/// towards the one transaction limit). After the active transaction is
/// completed, the session can immediately be re-used for the next transaction.
/// It is not necessary to create a new session for each transaction.
///
/// Transaction modes:
///
/// Cloud Spanner supports three transaction modes:
///
///    1. Locking read-write. This type of transaction is the only way
///       to write data into Cloud Spanner. These transactions rely on
///       pessimistic locking and, if necessary, two-phase commit.
///       Locking read-write transactions may abort, requiring the
///       application to retry.
///
///    2. Snapshot read-only. Snapshot read-only transactions provide guaranteed
///       consistency across several reads, but do not allow
///       writes. Snapshot read-only transactions can be configured to read at
///       timestamps in the past, or configured to perform a strong read
///       (where Spanner will select a timestamp such that the read is
///       guaranteed to see the effects of all transactions that have committed
///       before the start of the read). Snapshot read-only transactions do not
///       need to be committed.
///
///       Queries on change streams must be performed with the snapshot read-only
///       transaction mode, specifying a strong read. Please see
///       \[TransactionOptions.ReadOnly.strong][google.spanner.v1.TransactionOptions.ReadOnly.strong\]
///       for more details.
///
///    3. Partitioned DML. This type of transaction is used to execute
///       a single Partitioned DML statement. Partitioned DML partitions
///       the key space and runs the DML statement over each partition
///       in parallel using separate, internal transactions that commit
///       independently. Partitioned DML transactions do not need to be
///       committed.
///
/// For transactions that only read, snapshot read-only transactions
/// provide simpler semantics and are almost always faster. In
/// particular, read-only transactions do not take locks, so they do
/// not conflict with read-write transactions. As a consequence of not
/// taking locks, they also do not abort, so retry loops are not needed.
///
/// Transactions may only read-write data in a single database. They
/// may, however, read-write data in different tables within that
/// database.
///
/// Locking read-write transactions:
///
/// Locking transactions may be used to atomically read-modify-write
/// data anywhere in a database. This type of transaction is externally
/// consistent.
///
/// Clients should attempt to minimize the amount of time a transaction
/// is active. Faster transactions commit with higher probability
/// and cause less contention. Cloud Spanner attempts to keep read locks
/// active as long as the transaction continues to do reads, and the
/// transaction has not been terminated by
/// \[Commit][google.spanner.v1.Spanner.Commit\] or
/// \[Rollback][google.spanner.v1.Spanner.Rollback\]. Long periods of
/// inactivity at the client may cause Cloud Spanner to release a
/// transaction's locks and abort it.
///
/// Conceptually, a read-write transaction consists of zero or more
/// reads or SQL statements followed by
/// \[Commit][google.spanner.v1.Spanner.Commit\]. At any time before
/// \[Commit][google.spanner.v1.Spanner.Commit\], the client can send a
/// \[Rollback][google.spanner.v1.Spanner.Rollback\] request to abort the
/// transaction.
///
/// Semantics:
///
/// Cloud Spanner can commit the transaction if all read locks it acquired
/// are still valid at commit time, and it is able to acquire write
/// locks for all writes. Cloud Spanner can abort the transaction for any
/// reason. If a commit attempt returns `ABORTED`, Cloud Spanner guarantees
/// that the transaction has not modified any user data in Cloud Spanner.
///
/// Unless the transaction commits, Cloud Spanner makes no guarantees about
/// how long the transaction's locks were held for. It is an error to
/// use Cloud Spanner locks for any sort of mutual exclusion other than
/// between Cloud Spanner transactions themselves.
///
/// Retrying aborted transactions:
///
/// When a transaction aborts, the application can choose to retry the
/// whole transaction again. To maximize the chances of successfully
/// committing the retry, the client should execute the retry in the
/// same session as the original attempt. The original session's lock
/// priority increases with each consecutive abort, meaning that each
/// attempt has a slightly better chance of success than the previous.
///
/// Under some circumstances (for example, many transactions attempting to
/// modify the same row(s)), a transaction can abort many times in a
/// short period before successfully committing. Thus, it is not a good
/// idea to cap the number of retries a transaction can attempt;
/// instead, it is better to limit the total amount of time spent
/// retrying.
///
/// Idle transactions:
///
/// A transaction is considered idle if it has no outstanding reads or
/// SQL queries and has not started a read or SQL query within the last 10
/// seconds. Idle transactions can be aborted by Cloud Spanner so that they
/// don't hold on to locks indefinitely. If an idle transaction is aborted, the
/// commit will fail with error `ABORTED`.
///
/// If this behavior is undesirable, periodically executing a simple
/// SQL query in the transaction (for example, `SELECT 1`) prevents the
/// transaction from becoming idle.
///
/// Snapshot read-only transactions:
///
/// Snapshot read-only transactions provides a simpler method than
/// locking read-write transactions for doing several consistent
/// reads. However, this type of transaction does not support writes.
///
/// Snapshot transactions do not take locks. Instead, they work by
/// choosing a Cloud Spanner timestamp, then executing all reads at that
/// timestamp. Since they do not acquire locks, they do not block
/// concurrent read-write transactions.
///
/// Unlike locking read-write transactions, snapshot read-only
/// transactions never abort. They can fail if the chosen read
/// timestamp is garbage collected; however, the default garbage
/// collection policy is generous enough that most applications do not
/// need to worry about this in practice.
///
/// Snapshot read-only transactions do not need to call
/// \[Commit][google.spanner.v1.Spanner.Commit\] or
/// \[Rollback][google.spanner.v1.Spanner.Rollback\] (and in fact are not
/// permitted to do so).
///
/// To execute a snapshot transaction, the client specifies a timestamp
/// bound, which tells Cloud Spanner how to choose a read timestamp.
///
/// The types of timestamp bound are:
///
///    - Strong (the default).
///    - Bounded staleness.
///    - Exact staleness.
///
/// If the Cloud Spanner database to be read is geographically distributed,
/// stale read-only transactions can execute more quickly than strong
/// or read-write transactions, because they are able to execute far
/// from the leader replica.
///
/// Each type of timestamp bound is discussed in detail below.
///
/// Strong: Strong reads are guaranteed to see the effects of all transactions
/// that have committed before the start of the read. Furthermore, all
/// rows yielded by a single read are consistent with each other -- if
/// any part of the read observes a transaction, all parts of the read
/// see the transaction.
///
/// Strong reads are not repeatable: two consecutive strong read-only
/// transactions might return inconsistent results if there are
/// concurrent writes. If consistency across reads is required, the
/// reads should be executed within a transaction or at an exact read
/// timestamp.
///
/// Queries on change streams (see below for more details) must also specify
/// the strong read timestamp bound.
///
/// See
/// \[TransactionOptions.ReadOnly.strong][google.spanner.v1.TransactionOptions.ReadOnly.strong\].
///
/// Exact staleness:
///
/// These timestamp bounds execute reads at a user-specified
/// timestamp. Reads at a timestamp are guaranteed to see a consistent
/// prefix of the global transaction history: they observe
/// modifications done by all transactions with a commit timestamp less than or
/// equal to the read timestamp, and observe none of the modifications done by
/// transactions with a larger commit timestamp. They will block until
/// all conflicting transactions that may be assigned commit timestamps
/// <= the read timestamp have finished.
///
/// The timestamp can either be expressed as an absolute Cloud Spanner commit
/// timestamp or a staleness relative to the current time.
///
/// These modes do not require a "negotiation phase" to pick a
/// timestamp. As a result, they execute slightly faster than the
/// equivalent boundedly stale concurrency modes. On the other hand,
/// boundedly stale reads usually return fresher results.
///
/// See
/// \[TransactionOptions.ReadOnly.read_timestamp][google.spanner.v1.TransactionOptions.ReadOnly.read_timestamp\]
/// and
/// \[TransactionOptions.ReadOnly.exact_staleness][google.spanner.v1.TransactionOptions.ReadOnly.exact_staleness\].
///
/// Bounded staleness:
///
/// Bounded staleness modes allow Cloud Spanner to pick the read timestamp,
/// subject to a user-provided staleness bound. Cloud Spanner chooses the
/// newest timestamp within the staleness bound that allows execution
/// of the reads at the closest available replica without blocking.
///
/// All rows yielded are consistent with each other -- if any part of
/// the read observes a transaction, all parts of the read see the
/// transaction. Boundedly stale reads are not repeatable: two stale
/// reads, even if they use the same staleness bound, can execute at
/// different timestamps and thus return inconsistent results.
///
/// Boundedly stale reads execute in two phases: the first phase
/// negotiates a timestamp among all replicas needed to serve the
/// read. In the second phase, reads are executed at the negotiated
/// timestamp.
///
/// As a result of the two phase execution, bounded staleness reads are
/// usually a little slower than comparable exact staleness
/// reads. However, they are typically able to return fresher
/// results, and are more likely to execute at the closest replica.
///
/// Because the timestamp negotiation requires up-front knowledge of
/// which rows will be read, it can only be used with single-use
/// read-only transactions.
///
/// See
/// \[TransactionOptions.ReadOnly.max_staleness][google.spanner.v1.TransactionOptions.ReadOnly.max_staleness\]
/// and
/// \[TransactionOptions.ReadOnly.min_read_timestamp][google.spanner.v1.TransactionOptions.ReadOnly.min_read_timestamp\].
///
/// Old read timestamps and garbage collection:
///
/// Cloud Spanner continuously garbage collects deleted and overwritten data
/// in the background to reclaim storage space. This process is known
/// as "version GC". By default, version GC reclaims versions after they
/// are one hour old. Because of this, Cloud Spanner cannot perform reads
/// at read timestamps more than one hour in the past. This
/// restriction also applies to in-progress reads and/or SQL queries whose
/// timestamp become too old while executing. Reads and SQL queries with
/// too-old read timestamps fail with the error `FAILED_PRECONDITION`.
///
/// You can configure and extend the `VERSION_RETENTION_PERIOD` of a
/// database up to a period as long as one week, which allows Cloud Spanner
/// to perform reads up to one week in the past.
///
/// Querying change Streams:
///
/// A Change Stream is a schema object that can be configured to watch data
/// changes on the entire database, a set of tables, or a set of columns
/// in a database.
///
/// When a change stream is created, Spanner automatically defines a
/// corresponding SQL Table-Valued Function (TVF) that can be used to query
/// the change records in the associated change stream using the
/// ExecuteStreamingSql API. The name of the TVF for a change stream is
/// generated from the name of the change stream: READ_<change_stream_name>.
///
/// All queries on change stream TVFs must be executed using the
/// ExecuteStreamingSql API with a single-use read-only transaction with a
/// strong read-only timestamp_bound. The change stream TVF allows users to
/// specify the start_timestamp and end_timestamp for the time range of
/// interest. All change records within the retention period is accessible
/// using the strong read-only timestamp_bound. All other TransactionOptions
/// are invalid for change stream queries.
///
/// In addition, if TransactionOptions.read_only.return_read_timestamp is set
/// to true, a special value of 2^63 - 2 will be returned in the
/// \[Transaction][google.spanner.v1.Transaction\] message that describes the
/// transaction, instead of a valid read timestamp. This special value should be
/// discarded and not used for any subsequent queries.
///
/// Please see <https://cloud.google.com/spanner/docs/change-streams>
/// for more details on how to query the change stream TVFs.
///
/// Partitioned DML transactions:
///
/// Partitioned DML transactions are used to execute DML statements with a
/// different execution strategy that provides different, and often better,
/// scalability properties for large, table-wide operations than DML in a
/// ReadWrite transaction. Smaller scoped statements, such as an OLTP workload,
/// should prefer using ReadWrite transactions.
///
/// Partitioned DML partitions the keyspace and runs the DML statement on each
/// partition in separate, internal transactions. These transactions commit
/// automatically when complete, and run independently from one another.
///
/// To reduce lock contention, this execution strategy only acquires read locks
/// on rows that match the WHERE clause of the statement. Additionally, the
/// smaller per-partition transactions hold locks for less time.
///
/// That said, Partitioned DML is not a drop-in replacement for standard DML used
/// in ReadWrite transactions.
///
///   - The DML statement must be fully-partitionable. Specifically, the statement
///     must be expressible as the union of many statements which each access only
///     a single row of the table.
///
///   - The statement is not applied atomically to all rows of the table. Rather,
///     the statement is applied atomically to partitions of the table, in
///     independent transactions. Secondary index rows are updated atomically
///     with the base table rows.
///
///   - Partitioned DML does not guarantee exactly-once execution semantics
///     against a partition. The statement will be applied at least once to each
///     partition. It is strongly recommended that the DML statement should be
///     idempotent to avoid unexpected results. For instance, it is potentially
///     dangerous to run a statement such as
///     `UPDATE table SET column = column + 1` as it could be run multiple times
///     against some rows.
///
///   - The partitions are committed automatically - there is no support for
///     Commit or Rollback. If the call returns an error, or if the client issuing
///     the ExecuteSql call dies, it is possible that some rows had the statement
///     executed on them successfully. It is also possible that statement was
///     never executed against other rows.
///
///   - Partitioned DML transactions may only contain the execution of a single
///     DML statement via ExecuteSql or ExecuteStreamingSql.
///
///   - If any error is encountered during the execution of the partitioned DML
///     operation (for instance, a UNIQUE INDEX violation, division by zero, or a
///     value that cannot be stored due to schema constraints), then the
///     operation is stopped at that point and an error is returned. It is
///     possible that at this point, some partitions have been committed (or even
///     committed multiple times), and other partitions have not been run at all.
///
/// Given the above, Partitioned DML is good fit for large, database-wide,
/// operations that are idempotent, such as deleting old rows from a very large
/// table.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionOptions {
    /// Required. The type of transaction.
    #[prost(oneof = "transaction_options::Mode", tags = "1, 3, 2")]
    pub mode: ::core::option::Option<transaction_options::Mode>,
}
/// Nested message and enum types in `TransactionOptions`.
pub mod transaction_options {
    /// Message type to initiate a read-write transaction. Currently this
    /// transaction type has no options.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ReadWrite {
        /// Read lock mode for the transaction.
        #[prost(enumeration = "read_write::ReadLockMode", tag = "1")]
        pub read_lock_mode: i32,
    }
    /// Nested message and enum types in `ReadWrite`.
    pub mod read_write {
        /// `ReadLockMode` is used to set the read lock mode for read-write
        /// transactions.
        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            ::prost::Enumeration
        )]
        #[repr(i32)]
        pub enum ReadLockMode {
            /// Default value.
            ///
            /// If the value is not specified, the pessimistic read lock is used.
            Unspecified = 0,
            /// Pessimistic lock mode.
            ///
            /// Read locks are acquired immediately on read.
            Pessimistic = 1,
            /// Optimistic lock mode.
            ///
            /// Locks for reads within the transaction are not acquired on read.
            /// Instead the locks are acquired on a commit to validate that
            /// read/queried data has not changed since the transaction started.
            Optimistic = 2,
        }
        impl ReadLockMode {
            /// String value of the enum field names used in the ProtoBuf definition.
            ///
            /// The values are not transformed in any way and thus are considered stable
            /// (if the ProtoBuf definition does not change) and safe for programmatic use.
            pub fn as_str_name(&self) -> &'static str {
                match self {
                    ReadLockMode::Unspecified => "READ_LOCK_MODE_UNSPECIFIED",
                    ReadLockMode::Pessimistic => "PESSIMISTIC",
                    ReadLockMode::Optimistic => "OPTIMISTIC",
                }
            }
            /// Creates an enum from field names used in the ProtoBuf definition.
            pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
                match value {
                    "READ_LOCK_MODE_UNSPECIFIED" => Some(Self::Unspecified),
                    "PESSIMISTIC" => Some(Self::Pessimistic),
                    "OPTIMISTIC" => Some(Self::Optimistic),
                    _ => None,
                }
            }
        }
    }
    /// Message type to initiate a Partitioned DML transaction.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PartitionedDml {}
    /// Message type to initiate a read-only transaction.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ReadOnly {
        /// If true, the Cloud Spanner-selected read timestamp is included in
        /// the \[Transaction][google.spanner.v1.Transaction\] message that describes
        /// the transaction.
        #[prost(bool, tag = "6")]
        pub return_read_timestamp: bool,
        /// How to choose the timestamp for the read-only transaction.
        #[prost(oneof = "read_only::TimestampBound", tags = "1, 2, 3, 4, 5")]
        pub timestamp_bound: ::core::option::Option<read_only::TimestampBound>,
    }
    /// Nested message and enum types in `ReadOnly`.
    pub mod read_only {
        /// How to choose the timestamp for the read-only transaction.
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum TimestampBound {
            /// Read at a timestamp where all previously committed transactions
            /// are visible.
            #[prost(bool, tag = "1")]
            Strong(bool),
            /// Executes all reads at a timestamp >= `min_read_timestamp`.
            ///
            /// This is useful for requesting fresher data than some previous
            /// read, or data that is fresh enough to observe the effects of some
            /// previously committed transaction whose timestamp is known.
            ///
            /// Note that this option can only be used in single-use transactions.
            ///
            /// A timestamp in RFC3339 UTC \"Zulu\" format, accurate to nanoseconds.
            /// Example: `"2014-10-02T15:01:23.045123456Z"`.
            #[prost(message, tag = "2")]
            MinReadTimestamp(::prost_types::Timestamp),
            /// Read data at a timestamp >= `NOW - max_staleness`
            /// seconds. Guarantees that all writes that have committed more
            /// than the specified number of seconds ago are visible. Because
            /// Cloud Spanner chooses the exact timestamp, this mode works even if
            /// the client's local clock is substantially skewed from Cloud Spanner
            /// commit timestamps.
            ///
            /// Useful for reading the freshest data available at a nearby
            /// replica, while bounding the possible staleness if the local
            /// replica has fallen behind.
            ///
            /// Note that this option can only be used in single-use
            /// transactions.
            #[prost(message, tag = "3")]
            MaxStaleness(::prost_types::Duration),
            /// Executes all reads at the given timestamp. Unlike other modes,
            /// reads at a specific timestamp are repeatable; the same read at
            /// the same timestamp always returns the same data. If the
            /// timestamp is in the future, the read will block until the
            /// specified timestamp, modulo the read's deadline.
            ///
            /// Useful for large scale consistent reads such as mapreduces, or
            /// for coordinating many reads against a consistent snapshot of the
            /// data.
            ///
            /// A timestamp in RFC3339 UTC \"Zulu\" format, accurate to nanoseconds.
            /// Example: `"2014-10-02T15:01:23.045123456Z"`.
            #[prost(message, tag = "4")]
            ReadTimestamp(::prost_types::Timestamp),
            /// Executes all reads at a timestamp that is `exact_staleness`
            /// old. The timestamp is chosen soon after the read is started.
            ///
            /// Guarantees that all writes that have committed more than the
            /// specified number of seconds ago are visible. Because Cloud Spanner
            /// chooses the exact timestamp, this mode works even if the client's
            /// local clock is substantially skewed from Cloud Spanner commit
            /// timestamps.
            ///
            /// Useful for reading at nearby replicas without the distributed
            /// timestamp negotiation overhead of `max_staleness`.
            #[prost(message, tag = "5")]
            ExactStaleness(::prost_types::Duration),
        }
    }
    /// Required. The type of transaction.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Mode {
        /// Transaction may write.
        ///
        /// Authorization to begin a read-write transaction requires
        /// `spanner.databases.beginOrRollbackReadWriteTransaction` permission
        /// on the `session` resource.
        #[prost(message, tag = "1")]
        ReadWrite(ReadWrite),
        /// Partitioned DML transaction.
        ///
        /// Authorization to begin a Partitioned DML transaction requires
        /// `spanner.databases.beginPartitionedDmlTransaction` permission
        /// on the `session` resource.
        #[prost(message, tag = "3")]
        PartitionedDml(PartitionedDml),
        /// Transaction will not write.
        ///
        /// Authorization to begin a read-only transaction requires
        /// `spanner.databases.beginReadOnlyTransaction` permission
        /// on the `session` resource.
        #[prost(message, tag = "2")]
        ReadOnly(ReadOnly),
    }
}
/// A transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    /// `id` may be used to identify the transaction in subsequent
    /// \[Read][google.spanner.v1.Spanner.Read\],
    /// \[ExecuteSql][google.spanner.v1.Spanner.ExecuteSql\],
    /// \[Commit][google.spanner.v1.Spanner.Commit\], or
    /// \[Rollback][google.spanner.v1.Spanner.Rollback\] calls.
    ///
    /// Single-use read-only transactions do not have IDs, because
    /// single-use transactions do not support multiple requests.
    #[prost(bytes = "bytes", tag = "1")]
    pub id: ::prost::bytes::Bytes,
    /// For snapshot read-only transactions, the read timestamp chosen
    /// for the transaction. Not returned by default: see
    /// \[TransactionOptions.ReadOnly.return_read_timestamp][google.spanner.v1.TransactionOptions.ReadOnly.return_read_timestamp\].
    ///
    /// A timestamp in RFC3339 UTC \"Zulu\" format, accurate to nanoseconds.
    /// Example: `"2014-10-02T15:01:23.045123456Z"`.
    #[prost(message, optional, tag = "2")]
    pub read_timestamp: ::core::option::Option<::prost_types::Timestamp>,
}
/// This message is used to select the transaction in which a
/// \[Read][google.spanner.v1.Spanner.Read\] or
/// \[ExecuteSql][google.spanner.v1.Spanner.ExecuteSql\] call runs.
///
/// See \[TransactionOptions][google.spanner.v1.TransactionOptions\] for more
/// information about transactions.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionSelector {
    /// If no fields are set, the default is a single use transaction
    /// with strong concurrency.
    #[prost(oneof = "transaction_selector::Selector", tags = "1, 2, 3")]
    pub selector: ::core::option::Option<transaction_selector::Selector>,
}
/// Nested message and enum types in `TransactionSelector`.
pub mod transaction_selector {
    /// If no fields are set, the default is a single use transaction
    /// with strong concurrency.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Selector {
        /// Execute the read or SQL query in a temporary transaction.
        /// This is the most efficient way to execute a transaction that
        /// consists of a single SQL query.
        #[prost(message, tag = "1")]
        SingleUse(super::TransactionOptions),
        /// Execute the read or SQL query in a previously-started transaction.
        #[prost(bytes, tag = "2")]
        Id(::prost::bytes::Bytes),
        /// Begin a new transaction and execute this read or SQL query in
        /// it. The transaction ID of the new transaction is returned in
        /// \[ResultSetMetadata.transaction][google.spanner.v1.ResultSetMetadata.transaction\],
        /// which is a \[Transaction][google.spanner.v1.Transaction\].
        #[prost(message, tag = "3")]
        Begin(super::TransactionOptions),
    }
}
/// `Type` indicates the type of a Cloud Spanner value, as might be stored in a
/// table cell or returned from an SQL query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Type {
    /// Required. The \[TypeCode][google.spanner.v1.TypeCode\] for this type.
    #[prost(enumeration = "TypeCode", tag = "1")]
    pub code: i32,
    /// If \[code][google.spanner.v1.Type.code\] == \[ARRAY][google.spanner.v1.TypeCode.ARRAY\], then `array_element_type`
    /// is the type of the array elements.
    #[prost(message, optional, boxed, tag = "2")]
    pub array_element_type: ::core::option::Option<::prost::alloc::boxed::Box<Type>>,
    /// If \[code][google.spanner.v1.Type.code\] == \[STRUCT][google.spanner.v1.TypeCode.STRUCT\], then `struct_type`
    /// provides type information for the struct's fields.
    #[prost(message, optional, tag = "3")]
    pub struct_type: ::core::option::Option<StructType>,
    /// The \[TypeAnnotationCode][google.spanner.v1.TypeAnnotationCode\] that disambiguates SQL type that Spanner will
    /// use to represent values of this type during query processing. This is
    /// necessary for some type codes because a single \[TypeCode][google.spanner.v1.TypeCode\] can be mapped
    /// to different SQL types depending on the SQL dialect. \[type_annotation][google.spanner.v1.Type.type_annotation\]
    /// typically is not needed to process the content of a value (it doesn't
    /// affect serialization) and clients can ignore it on the read path.
    #[prost(enumeration = "TypeAnnotationCode", tag = "4")]
    pub type_annotation: i32,
}
/// `StructType` defines the fields of a \[STRUCT][google.spanner.v1.TypeCode.STRUCT\] type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StructType {
    /// The list of fields that make up this struct. Order is
    /// significant, because values of this struct type are represented as
    /// lists, where the order of field values matches the order of
    /// fields in the \[StructType][google.spanner.v1.StructType\]. In turn, the order of fields
    /// matches the order of columns in a read request, or the order of
    /// fields in the `SELECT` clause of a query.
    #[prost(message, repeated, tag = "1")]
    pub fields: ::prost::alloc::vec::Vec<struct_type::Field>,
}
/// Nested message and enum types in `StructType`.
pub mod struct_type {
    /// Message representing a single field of a struct.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Field {
        /// The name of the field. For reads, this is the column name. For
        /// SQL queries, it is the column alias (e.g., `"Word"` in the
        /// query `"SELECT 'hello' AS Word"`), or the column name (e.g.,
        /// `"ColName"` in the query `"SELECT ColName FROM Table"`). Some
        /// columns might have an empty name (e.g., `"SELECT
        /// UPPER(ColName)"`). Note that a query result can contain
        /// multiple fields with the same name.
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
        /// The type of the field.
        #[prost(message, optional, tag = "2")]
        pub r#type: ::core::option::Option<super::Type>,
    }
}
/// `TypeCode` is used as part of \[Type][google.spanner.v1.Type\] to
/// indicate the type of a Cloud Spanner value.
///
/// Each legal value of a type can be encoded to or decoded from a JSON
/// value, using the encodings described below. All Cloud Spanner values can
/// be `null`, regardless of type; `null`s are always encoded as a JSON
/// `null`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TypeCode {
    /// Not specified.
    Unspecified = 0,
    /// Encoded as JSON `true` or `false`.
    Bool = 1,
    /// Encoded as `string`, in decimal format.
    Int64 = 2,
    /// Encoded as `number`, or the strings `"NaN"`, `"Infinity"`, or
    /// `"-Infinity"`.
    Float64 = 3,
    /// Encoded as `string` in RFC 3339 timestamp format. The time zone
    /// must be present, and must be `"Z"`.
    ///
    /// If the schema has the column option
    /// `allow_commit_timestamp=true`, the placeholder string
    /// `"spanner.commit_timestamp()"` can be used to instruct the system
    /// to insert the commit timestamp associated with the transaction
    /// commit.
    Timestamp = 4,
    /// Encoded as `string` in RFC 3339 date format.
    Date = 5,
    /// Encoded as `string`.
    String = 6,
    /// Encoded as a base64-encoded `string`, as described in RFC 4648,
    /// section 4.
    Bytes = 7,
    /// Encoded as `list`, where the list elements are represented
    /// according to
    /// \[array_element_type][google.spanner.v1.Type.array_element_type\].
    Array = 8,
    /// Encoded as `list`, where list element `i` is represented according
    /// to \[struct_type.fields[i]][google.spanner.v1.StructType.fields\].
    Struct = 9,
    /// Encoded as `string`, in decimal format or scientific notation format.
    /// <br>Decimal format:
    /// <br>`\[+-]Digits[.[Digits]\]` or
    /// <br>`\[+-][Digits\].Digits`
    ///
    /// Scientific notation:
    /// <br>`\[+-]Digits[.[Digits]][ExponentIndicator[+-]Digits\]` or
    /// <br>`\[+-][Digits].Digits[ExponentIndicator[+-]Digits\]`
    /// <br>(ExponentIndicator is `"e"` or `"E"`)
    Numeric = 10,
    /// Encoded as a JSON-formatted `string` as described in RFC 7159. The
    /// following rules are applied when parsing JSON input:
    ///
    /// - Whitespace characters are not preserved.
    /// - If a JSON object has duplicate keys, only the first key is preserved.
    /// - Members of a JSON object are not guaranteed to have their order
    ///    preserved.
    /// - JSON array elements will have their order preserved.
    Json = 11,
}
impl TypeCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TypeCode::Unspecified => "TYPE_CODE_UNSPECIFIED",
            TypeCode::Bool => "BOOL",
            TypeCode::Int64 => "INT64",
            TypeCode::Float64 => "FLOAT64",
            TypeCode::Timestamp => "TIMESTAMP",
            TypeCode::Date => "DATE",
            TypeCode::String => "STRING",
            TypeCode::Bytes => "BYTES",
            TypeCode::Array => "ARRAY",
            TypeCode::Struct => "STRUCT",
            TypeCode::Numeric => "NUMERIC",
            TypeCode::Json => "JSON",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TYPE_CODE_UNSPECIFIED" => Some(Self::Unspecified),
            "BOOL" => Some(Self::Bool),
            "INT64" => Some(Self::Int64),
            "FLOAT64" => Some(Self::Float64),
            "TIMESTAMP" => Some(Self::Timestamp),
            "DATE" => Some(Self::Date),
            "STRING" => Some(Self::String),
            "BYTES" => Some(Self::Bytes),
            "ARRAY" => Some(Self::Array),
            "STRUCT" => Some(Self::Struct),
            "NUMERIC" => Some(Self::Numeric),
            "JSON" => Some(Self::Json),
            _ => None,
        }
    }
}
/// `TypeAnnotationCode` is used as a part of \[Type][google.spanner.v1.Type\] to
/// disambiguate SQL types that should be used for a given Cloud Spanner value.
/// Disambiguation is needed because the same Cloud Spanner type can be mapped to
/// different SQL types depending on SQL dialect. TypeAnnotationCode doesn't
/// affect the way value is serialized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TypeAnnotationCode {
    /// Not specified.
    Unspecified = 0,
    /// PostgreSQL compatible NUMERIC type. This annotation needs to be applied to
    /// \[Type][google.spanner.v1.Type\] instances having \[NUMERIC][google.spanner.v1.TypeCode.NUMERIC\]
    /// type code to specify that values of this type should be treated as
    /// PostgreSQL NUMERIC values. Currently this annotation is always needed for
    /// \[NUMERIC][google.spanner.v1.TypeCode.NUMERIC\] when a client interacts with PostgreSQL-enabled
    /// Spanner databases.
    PgNumeric = 2,
    /// PostgreSQL compatible JSONB type. This annotation needs to be applied to
    /// \[Type][google.spanner.v1.Type\] instances having \[JSON][google.spanner.v1.TypeCode.JSON\]
    /// type code to specify that values of this type should be treated as
    /// PostgreSQL JSONB values. Currently this annotation is always needed for
    /// \[JSON][google.spanner.v1.TypeCode.JSON\] when a client interacts with PostgreSQL-enabled
    /// Spanner databases.
    PgJsonb = 3,
}
impl TypeAnnotationCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TypeAnnotationCode::Unspecified => "TYPE_ANNOTATION_CODE_UNSPECIFIED",
            TypeAnnotationCode::PgNumeric => "PG_NUMERIC",
            TypeAnnotationCode::PgJsonb => "PG_JSONB",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TYPE_ANNOTATION_CODE_UNSPECIFIED" => Some(Self::Unspecified),
            "PG_NUMERIC" => Some(Self::PgNumeric),
            "PG_JSONB" => Some(Self::PgJsonb),
            _ => None,
        }
    }
}
/// Results from \[Read][google.spanner.v1.Spanner.Read\] or
/// \[ExecuteSql][google.spanner.v1.Spanner.ExecuteSql\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResultSet {
    /// Metadata about the result set, such as row type information.
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<ResultSetMetadata>,
    /// Each element in `rows` is a row whose format is defined by
    /// \[metadata.row_type][google.spanner.v1.ResultSetMetadata.row_type\]. The ith element
    /// in each row matches the ith field in
    /// \[metadata.row_type][google.spanner.v1.ResultSetMetadata.row_type\]. Elements are
    /// encoded based on type as described
    /// \[here][google.spanner.v1.TypeCode\].
    #[prost(message, repeated, tag = "2")]
    pub rows: ::prost::alloc::vec::Vec<::prost_types::ListValue>,
    /// Query plan and execution statistics for the SQL statement that
    /// produced this result set. These can be requested by setting
    /// \[ExecuteSqlRequest.query_mode][google.spanner.v1.ExecuteSqlRequest.query_mode\].
    /// DML statements always produce stats containing the number of rows
    /// modified, unless executed using the
    /// \[ExecuteSqlRequest.QueryMode.PLAN][google.spanner.v1.ExecuteSqlRequest.QueryMode.PLAN\] \[ExecuteSqlRequest.query_mode][google.spanner.v1.ExecuteSqlRequest.query_mode\].
    /// Other fields may or may not be populated, based on the
    /// \[ExecuteSqlRequest.query_mode][google.spanner.v1.ExecuteSqlRequest.query_mode\].
    #[prost(message, optional, tag = "3")]
    pub stats: ::core::option::Option<ResultSetStats>,
}
/// Partial results from a streaming read or SQL query. Streaming reads and
/// SQL queries better tolerate large result sets, large rows, and large
/// values, but are a little trickier to consume.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartialResultSet {
    /// Metadata about the result set, such as row type information.
    /// Only present in the first response.
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<ResultSetMetadata>,
    /// A streamed result set consists of a stream of values, which might
    /// be split into many `PartialResultSet` messages to accommodate
    /// large rows and/or large values. Every N complete values defines a
    /// row, where N is equal to the number of entries in
    /// \[metadata.row_type.fields][google.spanner.v1.StructType.fields\].
    ///
    /// Most values are encoded based on type as described
    /// \[here][google.spanner.v1.TypeCode\].
    ///
    /// It is possible that the last value in values is "chunked",
    /// meaning that the rest of the value is sent in subsequent
    /// `PartialResultSet`(s). This is denoted by the \[chunked_value][google.spanner.v1.PartialResultSet.chunked_value\]
    /// field. Two or more chunked values can be merged to form a
    /// complete value as follows:
    ///
    ///    * `bool/number/null`: cannot be chunked
    ///    * `string`: concatenate the strings
    ///    * `list`: concatenate the lists. If the last element in a list is a
    ///      `string`, `list`, or `object`, merge it with the first element in
    ///      the next list by applying these rules recursively.
    ///    * `object`: concatenate the (field name, field value) pairs. If a
    ///      field name is duplicated, then apply these rules recursively
    ///      to merge the field values.
    ///
    /// Some examples of merging:
    ///
    ///      # Strings are concatenated.
    ///      "foo", "bar" => "foobar"
    ///
    ///      # Lists of non-strings are concatenated.
    ///      [2, 3], \[4\] => [2, 3, 4]
    ///
    ///      # Lists are concatenated, but the last and first elements are merged
    ///      # because they are strings.
    ///      ["a", "b"], ["c", "d"] => ["a", "bc", "d"]
    ///
    ///      # Lists are concatenated, but the last and first elements are merged
    ///      # because they are lists. Recursively, the last and first elements
    ///      # of the inner lists are merged because they are strings.
    ///      ["a", ["b", "c"]], \[["d"\], "e"] => ["a", ["b", "cd"], "e"]
    ///
    ///      # Non-overlapping object fields are combined.
    ///      {"a": "1"}, {"b": "2"} => {"a": "1", "b": 2"}
    ///
    ///      # Overlapping object fields are merged.
    ///      {"a": "1"}, {"a": "2"} => {"a": "12"}
    ///
    ///      # Examples of merging objects containing lists of strings.
    ///      {"a": \["1"\]}, {"a": \["2"\]} => {"a": \["12"\]}
    ///
    /// For a more complete example, suppose a streaming SQL query is
    /// yielding a result set whose rows contain a single string
    /// field. The following `PartialResultSet`s might be yielded:
    ///
    ///      {
    ///        "metadata": { ... }
    ///        "values": ["Hello", "W"]
    ///        "chunked_value": true
    ///        "resume_token": "Af65..."
    ///      }
    ///      {
    ///        "values": \["orl"\]
    ///        "chunked_value": true
    ///        "resume_token": "Bqp2..."
    ///      }
    ///      {
    ///        "values": \["d"\]
    ///        "resume_token": "Zx1B..."
    ///      }
    ///
    /// This sequence of `PartialResultSet`s encodes two rows, one
    /// containing the field value `"Hello"`, and a second containing the
    /// field value `"World" = "W" + "orl" + "d"`.
    #[prost(message, repeated, tag = "2")]
    pub values: ::prost::alloc::vec::Vec<::prost_types::Value>,
    /// If true, then the final value in \[values][google.spanner.v1.PartialResultSet.values\] is chunked, and must
    /// be combined with more values from subsequent `PartialResultSet`s
    /// to obtain a complete field value.
    #[prost(bool, tag = "3")]
    pub chunked_value: bool,
    /// Streaming calls might be interrupted for a variety of reasons, such
    /// as TCP connection loss. If this occurs, the stream of results can
    /// be resumed by re-sending the original request and including
    /// `resume_token`. Note that executing any other transaction in the
    /// same session invalidates the token.
    #[prost(bytes = "bytes", tag = "4")]
    pub resume_token: ::prost::bytes::Bytes,
    /// Query plan and execution statistics for the statement that produced this
    /// streaming result set. These can be requested by setting
    /// \[ExecuteSqlRequest.query_mode][google.spanner.v1.ExecuteSqlRequest.query_mode\] and are sent
    /// only once with the last response in the stream.
    /// This field will also be present in the last response for DML
    /// statements.
    #[prost(message, optional, tag = "5")]
    pub stats: ::core::option::Option<ResultSetStats>,
}
/// Metadata about a \[ResultSet][google.spanner.v1.ResultSet\] or \[PartialResultSet][google.spanner.v1.PartialResultSet\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResultSetMetadata {
    /// Indicates the field names and types for the rows in the result
    /// set.  For example, a SQL query like `"SELECT UserId, UserName FROM
    /// Users"` could return a `row_type` value like:
    ///
    ///      "fields": [
    ///        { "name": "UserId", "type": { "code": "INT64" } },
    ///        { "name": "UserName", "type": { "code": "STRING" } },
    ///      ]
    #[prost(message, optional, tag = "1")]
    pub row_type: ::core::option::Option<StructType>,
    /// If the read or SQL query began a transaction as a side-effect, the
    /// information about the new transaction is yielded here.
    #[prost(message, optional, tag = "2")]
    pub transaction: ::core::option::Option<Transaction>,
    /// A SQL query can be parameterized. In PLAN mode, these parameters can be
    /// undeclared. This indicates the field names and types for those undeclared
    /// parameters in the SQL query. For example, a SQL query like `"SELECT * FROM
    /// Users where UserId = @userId and UserName = @userName "` could return a
    /// `undeclared_parameters` value like:
    ///
    ///      "fields": [
    ///        { "name": "UserId", "type": { "code": "INT64" } },
    ///        { "name": "UserName", "type": { "code": "STRING" } },
    ///      ]
    #[prost(message, optional, tag = "3")]
    pub undeclared_parameters: ::core::option::Option<StructType>,
}
/// Additional statistics about a \[ResultSet][google.spanner.v1.ResultSet\] or \[PartialResultSet][google.spanner.v1.PartialResultSet\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResultSetStats {
    /// \[QueryPlan][google.spanner.v1.QueryPlan\] for the query associated with this result.
    #[prost(message, optional, tag = "1")]
    pub query_plan: ::core::option::Option<QueryPlan>,
    /// Aggregated statistics from the execution of the query. Only present when
    /// the query is profiled. For example, a query could return the statistics as
    /// follows:
    ///
    ///      {
    ///        "rows_returned": "3",
    ///        "elapsed_time": "1.22 secs",
    ///        "cpu_time": "1.19 secs"
    ///      }
    #[prost(message, optional, tag = "2")]
    pub query_stats: ::core::option::Option<::prost_types::Struct>,
    /// The number of rows modified by the DML statement.
    #[prost(oneof = "result_set_stats::RowCount", tags = "3, 4")]
    pub row_count: ::core::option::Option<result_set_stats::RowCount>,
}
/// Nested message and enum types in `ResultSetStats`.
pub mod result_set_stats {
    /// The number of rows modified by the DML statement.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum RowCount {
        /// Standard DML returns an exact count of rows that were modified.
        #[prost(int64, tag = "3")]
        RowCountExact(i64),
        /// Partitioned DML does not offer exactly-once semantics, so it
        /// returns a lower bound of the rows modified.
        #[prost(int64, tag = "4")]
        RowCountLowerBound(i64),
    }
}
/// The request for \[CreateSession][google.spanner.v1.Spanner.CreateSession\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSessionRequest {
    /// Required. The database in which the new session is created.
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
    /// Required. The session to create.
    #[prost(message, optional, tag = "2")]
    pub session: ::core::option::Option<Session>,
}
/// The request for \[BatchCreateSessions][google.spanner.v1.Spanner.BatchCreateSessions\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchCreateSessionsRequest {
    /// Required. The database in which the new sessions are created.
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
    /// Parameters to be applied to each created session.
    #[prost(message, optional, tag = "2")]
    pub session_template: ::core::option::Option<Session>,
    /// Required. The number of sessions to be created in this batch call.
    /// The API may return fewer than the requested number of sessions. If a
    /// specific number of sessions are desired, the client can make additional
    /// calls to BatchCreateSessions (adjusting
    /// \[session_count][google.spanner.v1.BatchCreateSessionsRequest.session_count\] as necessary).
    #[prost(int32, tag = "3")]
    pub session_count: i32,
}
/// The response for \[BatchCreateSessions][google.spanner.v1.Spanner.BatchCreateSessions\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchCreateSessionsResponse {
    /// The freshly created sessions.
    #[prost(message, repeated, tag = "1")]
    pub session: ::prost::alloc::vec::Vec<Session>,
}
/// A session in the Cloud Spanner API.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Session {
    /// Output only. The name of the session. This is always system-assigned.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The labels for the session.
    ///
    ///   * Label keys must be between 1 and 63 characters long and must conform to
    ///     the following regular expression: `\[a-z]([-a-z0-9]*[a-z0-9\])?`.
    ///   * Label values must be between 0 and 63 characters long and must conform
    ///     to the regular expression `(\[a-z]([-a-z0-9]*[a-z0-9\])?)?`.
    ///   * No more than 64 labels can be associated with a given session.
    ///
    /// See <https://goo.gl/xmQnxf> for more information on and examples of labels.
    #[prost(map = "string, string", tag = "2")]
    pub labels: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    /// Output only. The timestamp when the session is created.
    #[prost(message, optional, tag = "3")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The approximate timestamp when the session is last used. It is
    /// typically earlier than the actual last use time.
    #[prost(message, optional, tag = "4")]
    pub approximate_last_use_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The database role which created this session.
    #[prost(string, tag = "5")]
    pub creator_role: ::prost::alloc::string::String,
}
/// The request for \[GetSession][google.spanner.v1.Spanner.GetSession\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSessionRequest {
    /// Required. The name of the session to retrieve.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The request for \[ListSessions][google.spanner.v1.Spanner.ListSessions\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSessionsRequest {
    /// Required. The database in which to list sessions.
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
    /// Number of sessions to be returned in the response. If 0 or less, defaults
    /// to the server's maximum allowed page size.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.v1.ListSessionsResponse.next_page_token\] from a previous
    /// \[ListSessionsResponse][google.spanner.v1.ListSessionsResponse\].
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
    /// An expression for filtering the results of the request. Filter rules are
    /// case insensitive. The fields eligible for filtering are:
    ///
    ///    * `labels.key` where key is the name of a label
    ///
    /// Some examples of using filters are:
    ///
    ///    * `labels.env:*` --> The session has the label "env".
    ///    * `labels.env:dev` --> The session has the label "env" and the value of
    ///                         the label contains the string "dev".
    #[prost(string, tag = "4")]
    pub filter: ::prost::alloc::string::String,
}
/// The response for \[ListSessions][google.spanner.v1.Spanner.ListSessions\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSessionsResponse {
    /// The list of requested sessions.
    #[prost(message, repeated, tag = "1")]
    pub sessions: ::prost::alloc::vec::Vec<Session>,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListSessions][google.spanner.v1.Spanner.ListSessions\] call to fetch more of the matching
    /// sessions.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The request for \[DeleteSession][google.spanner.v1.Spanner.DeleteSession\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSessionRequest {
    /// Required. The name of the session to delete.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// Common request options for various APIs.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestOptions {
    /// Priority for the request.
    #[prost(enumeration = "request_options::Priority", tag = "1")]
    pub priority: i32,
    /// A per-request tag which can be applied to queries or reads, used for
    /// statistics collection.
    /// Both request_tag and transaction_tag can be specified for a read or query
    /// that belongs to a transaction.
    /// This field is ignored for requests where it's not applicable (e.g.
    /// CommitRequest).
    /// Legal characters for `request_tag` values are all printable characters
    /// (ASCII 32 - 126) and the length of a request_tag is limited to 50
    /// characters. Values that exceed this limit are truncated.
    /// Any leading underscore (_) characters will be removed from the string.
    #[prost(string, tag = "2")]
    pub request_tag: ::prost::alloc::string::String,
    /// A tag used for statistics collection about this transaction.
    /// Both request_tag and transaction_tag can be specified for a read or query
    /// that belongs to a transaction.
    /// The value of transaction_tag should be the same for all requests belonging
    /// to the same transaction.
    /// If this request doesn't belong to any transaction, transaction_tag will be
    /// ignored.
    /// Legal characters for `transaction_tag` values are all printable characters
    /// (ASCII 32 - 126) and the length of a transaction_tag is limited to 50
    /// characters. Values that exceed this limit are truncated.
    /// Any leading underscore (_) characters will be removed from the string.
    #[prost(string, tag = "3")]
    pub transaction_tag: ::prost::alloc::string::String,
}
/// Nested message and enum types in `RequestOptions`.
pub mod request_options {
    /// The relative priority for requests. Note that priority is not applicable
    /// for \[BeginTransaction][google.spanner.v1.Spanner.BeginTransaction\].
    ///
    /// The priority acts as a hint to the Cloud Spanner scheduler and does not
    /// guarantee priority or order of execution. For example:
    ///
    /// * Some parts of a write operation always execute at `PRIORITY_HIGH`,
    ///    regardless of the specified priority. This may cause you to see an
    ///    increase in high priority workload even when executing a low priority
    ///    request. This can also potentially cause a priority inversion where a
    ///    lower priority request will be fulfilled ahead of a higher priority
    ///    request.
    /// * If a transaction contains multiple operations with different priorities,
    ///    Cloud Spanner does not guarantee to process the higher priority
    ///    operations first. There may be other constraints to satisfy, such as
    ///    order of operations.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Priority {
        /// `PRIORITY_UNSPECIFIED` is equivalent to `PRIORITY_HIGH`.
        Unspecified = 0,
        /// This specifies that the request is low priority.
        Low = 1,
        /// This specifies that the request is medium priority.
        Medium = 2,
        /// This specifies that the request is high priority.
        High = 3,
    }
    impl Priority {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Priority::Unspecified => "PRIORITY_UNSPECIFIED",
                Priority::Low => "PRIORITY_LOW",
                Priority::Medium => "PRIORITY_MEDIUM",
                Priority::High => "PRIORITY_HIGH",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "PRIORITY_UNSPECIFIED" => Some(Self::Unspecified),
                "PRIORITY_LOW" => Some(Self::Low),
                "PRIORITY_MEDIUM" => Some(Self::Medium),
                "PRIORITY_HIGH" => Some(Self::High),
                _ => None,
            }
        }
    }
}
/// The request for \[ExecuteSql][google.spanner.v1.Spanner.ExecuteSql\] and
/// \[ExecuteStreamingSql][google.spanner.v1.Spanner.ExecuteStreamingSql\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecuteSqlRequest {
    /// Required. The session in which the SQL query should be performed.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// The transaction to use.
    ///
    /// For queries, if none is provided, the default is a temporary read-only
    /// transaction with strong concurrency.
    ///
    /// Standard DML statements require a read-write transaction. To protect
    /// against replays, single-use transactions are not supported.  The caller
    /// must either supply an existing transaction ID or begin a new transaction.
    ///
    /// Partitioned DML requires an existing Partitioned DML transaction ID.
    #[prost(message, optional, tag = "2")]
    pub transaction: ::core::option::Option<TransactionSelector>,
    /// Required. The SQL string.
    #[prost(string, tag = "3")]
    pub sql: ::prost::alloc::string::String,
    /// Parameter names and values that bind to placeholders in the SQL string.
    ///
    /// A parameter placeholder consists of the `@` character followed by the
    /// parameter name (for example, `@firstName`). Parameter names must conform
    /// to the naming requirements of identifiers as specified at
    /// <https://cloud.google.com/spanner/docs/lexical#identifiers.>
    ///
    /// Parameters can appear anywhere that a literal value is expected.  The same
    /// parameter name can be used more than once, for example:
    ///
    /// `"WHERE id > @msg_id AND id < @msg_id + 100"`
    ///
    /// It is an error to execute a SQL statement with unbound parameters.
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<::prost_types::Struct>,
    /// It is not always possible for Cloud Spanner to infer the right SQL type
    /// from a JSON value.  For example, values of type `BYTES` and values
    /// of type `STRING` both appear in \[params][google.spanner.v1.ExecuteSqlRequest.params\] as JSON strings.
    ///
    /// In these cases, `param_types` can be used to specify the exact
    /// SQL type for some or all of the SQL statement parameters. See the
    /// definition of \[Type][google.spanner.v1.Type\] for more information
    /// about SQL types.
    #[prost(map = "string, message", tag = "5")]
    pub param_types: ::std::collections::HashMap<::prost::alloc::string::String, Type>,
    /// If this request is resuming a previously interrupted SQL statement
    /// execution, `resume_token` should be copied from the last
    /// \[PartialResultSet][google.spanner.v1.PartialResultSet\] yielded before the interruption. Doing this
    /// enables the new SQL statement execution to resume where the last one left
    /// off. The rest of the request parameters must exactly match the
    /// request that yielded this token.
    #[prost(bytes = "bytes", tag = "6")]
    pub resume_token: ::prost::bytes::Bytes,
    /// Used to control the amount of debugging information returned in
    /// \[ResultSetStats][google.spanner.v1.ResultSetStats\]. If \[partition_token][google.spanner.v1.ExecuteSqlRequest.partition_token\] is set, \[query_mode][google.spanner.v1.ExecuteSqlRequest.query_mode\] can only
    /// be set to \[QueryMode.NORMAL][google.spanner.v1.ExecuteSqlRequest.QueryMode.NORMAL\].
    #[prost(enumeration = "execute_sql_request::QueryMode", tag = "7")]
    pub query_mode: i32,
    /// If present, results will be restricted to the specified partition
    /// previously created using PartitionQuery().  There must be an exact
    /// match for the values of fields common to this message and the
    /// PartitionQueryRequest message used to create this partition_token.
    #[prost(bytes = "bytes", tag = "8")]
    pub partition_token: ::prost::bytes::Bytes,
    /// A per-transaction sequence number used to identify this request. This field
    /// makes each request idempotent such that if the request is received multiple
    /// times, at most one will succeed.
    ///
    /// The sequence number must be monotonically increasing within the
    /// transaction. If a request arrives for the first time with an out-of-order
    /// sequence number, the transaction may be aborted. Replays of previously
    /// handled requests will yield the same response as the first execution.
    ///
    /// Required for DML statements. Ignored for queries.
    #[prost(int64, tag = "9")]
    pub seqno: i64,
    /// Query optimizer configuration to use for the given query.
    #[prost(message, optional, tag = "10")]
    pub query_options: ::core::option::Option<execute_sql_request::QueryOptions>,
    /// Common options for this request.
    #[prost(message, optional, tag = "11")]
    pub request_options: ::core::option::Option<RequestOptions>,
    /// If this is for a partitioned query and this field is set to `true`, the
    /// request will be executed via Spanner independent compute resources.
    ///
    /// If the field is set to `true` but the request does not set
    /// `partition_token`, the API will return an `INVALID_ARGUMENT` error.
    #[prost(bool, tag = "16")]
    pub data_boost_enabled: bool,
}
/// Nested message and enum types in `ExecuteSqlRequest`.
pub mod execute_sql_request {
    /// Query optimizer configuration.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct QueryOptions {
        /// An option to control the selection of optimizer version.
        ///
        /// This parameter allows individual queries to pick different query
        /// optimizer versions.
        ///
        /// Specifying `latest` as a value instructs Cloud Spanner to use the
        /// latest supported query optimizer version. If not specified, Cloud Spanner
        /// uses the optimizer version set at the database level options. Any other
        /// positive integer (from the list of supported optimizer versions)
        /// overrides the default optimizer version for query execution.
        ///
        /// The list of supported optimizer versions can be queried from
        /// SPANNER_SYS.SUPPORTED_OPTIMIZER_VERSIONS.
        ///
        /// Executing a SQL statement with an invalid optimizer version fails with
        /// an `INVALID_ARGUMENT` error.
        ///
        /// See
        /// <https://cloud.google.com/spanner/docs/query-optimizer/manage-query-optimizer>
        /// for more information on managing the query optimizer.
        ///
        /// The `optimizer_version` statement hint has precedence over this setting.
        #[prost(string, tag = "1")]
        pub optimizer_version: ::prost::alloc::string::String,
        /// An option to control the selection of optimizer statistics package.
        ///
        /// This parameter allows individual queries to use a different query
        /// optimizer statistics package.
        ///
        /// Specifying `latest` as a value instructs Cloud Spanner to use the latest
        /// generated statistics package. If not specified, Cloud Spanner uses
        /// the statistics package set at the database level options, or the latest
        /// package if the database option is not set.
        ///
        /// The statistics package requested by the query has to be exempt from
        /// garbage collection. This can be achieved with the following DDL
        /// statement:
        ///
        /// ```
        /// ALTER STATISTICS <package_name> SET OPTIONS (allow_gc=false)
        /// ```
        ///
        /// The list of available statistics packages can be queried from
        /// `INFORMATION_SCHEMA.SPANNER_STATISTICS`.
        ///
        /// Executing a SQL statement with an invalid optimizer statistics package
        /// or with a statistics package that allows garbage collection fails with
        /// an `INVALID_ARGUMENT` error.
        #[prost(string, tag = "2")]
        pub optimizer_statistics_package: ::prost::alloc::string::String,
    }
    /// Mode in which the statement must be processed.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum QueryMode {
        /// The default mode. Only the statement results are returned.
        Normal = 0,
        /// This mode returns only the query plan, without any results or
        /// execution statistics information.
        Plan = 1,
        /// This mode returns both the query plan and the execution statistics along
        /// with the results.
        Profile = 2,
    }
    impl QueryMode {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                QueryMode::Normal => "NORMAL",
                QueryMode::Plan => "PLAN",
                QueryMode::Profile => "PROFILE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "NORMAL" => Some(Self::Normal),
                "PLAN" => Some(Self::Plan),
                "PROFILE" => Some(Self::Profile),
                _ => None,
            }
        }
    }
}
/// The request for \[ExecuteBatchDml][google.spanner.v1.Spanner.ExecuteBatchDml\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecuteBatchDmlRequest {
    /// Required. The session in which the DML statements should be performed.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// Required. The transaction to use. Must be a read-write transaction.
    ///
    /// To protect against replays, single-use transactions are not supported. The
    /// caller must either supply an existing transaction ID or begin a new
    /// transaction.
    #[prost(message, optional, tag = "2")]
    pub transaction: ::core::option::Option<TransactionSelector>,
    /// Required. The list of statements to execute in this batch. Statements are executed
    /// serially, such that the effects of statement `i` are visible to statement
    /// `i+1`. Each statement must be a DML statement. Execution stops at the
    /// first failed statement; the remaining statements are not executed.
    ///
    /// Callers must provide at least one statement.
    #[prost(message, repeated, tag = "3")]
    pub statements: ::prost::alloc::vec::Vec<execute_batch_dml_request::Statement>,
    /// Required. A per-transaction sequence number used to identify this request. This field
    /// makes each request idempotent such that if the request is received multiple
    /// times, at most one will succeed.
    ///
    /// The sequence number must be monotonically increasing within the
    /// transaction. If a request arrives for the first time with an out-of-order
    /// sequence number, the transaction may be aborted. Replays of previously
    /// handled requests will yield the same response as the first execution.
    #[prost(int64, tag = "4")]
    pub seqno: i64,
    /// Common options for this request.
    #[prost(message, optional, tag = "5")]
    pub request_options: ::core::option::Option<RequestOptions>,
}
/// Nested message and enum types in `ExecuteBatchDmlRequest`.
pub mod execute_batch_dml_request {
    /// A single DML statement.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Statement {
        /// Required. The DML string.
        #[prost(string, tag = "1")]
        pub sql: ::prost::alloc::string::String,
        /// Parameter names and values that bind to placeholders in the DML string.
        ///
        /// A parameter placeholder consists of the `@` character followed by the
        /// parameter name (for example, `@firstName`). Parameter names can contain
        /// letters, numbers, and underscores.
        ///
        /// Parameters can appear anywhere that a literal value is expected.  The
        /// same parameter name can be used more than once, for example:
        ///
        /// `"WHERE id > @msg_id AND id < @msg_id + 100"`
        ///
        /// It is an error to execute a SQL statement with unbound parameters.
        #[prost(message, optional, tag = "2")]
        pub params: ::core::option::Option<::prost_types::Struct>,
        /// It is not always possible for Cloud Spanner to infer the right SQL type
        /// from a JSON value.  For example, values of type `BYTES` and values
        /// of type `STRING` both appear in \[params][google.spanner.v1.ExecuteBatchDmlRequest.Statement.params\] as JSON strings.
        ///
        /// In these cases, `param_types` can be used to specify the exact
        /// SQL type for some or all of the SQL statement parameters. See the
        /// definition of \[Type][google.spanner.v1.Type\] for more information
        /// about SQL types.
        #[prost(map = "string, message", tag = "3")]
        pub param_types: ::std::collections::HashMap<
            ::prost::alloc::string::String,
            super::Type,
        >,
    }
}
/// The response for \[ExecuteBatchDml][google.spanner.v1.Spanner.ExecuteBatchDml\]. Contains a list
/// of \[ResultSet][google.spanner.v1.ResultSet\] messages, one for each DML statement that has successfully
/// executed, in the same order as the statements in the request. If a statement
/// fails, the status in the response body identifies the cause of the failure.
///
/// To check for DML statements that failed, use the following approach:
///
/// 1. Check the status in the response message. The \[google.rpc.Code][google.rpc.Code\] enum
///     value `OK` indicates that all statements were executed successfully.
/// 2. If the status was not `OK`, check the number of result sets in the
///     response. If the response contains `N` \[ResultSet][google.spanner.v1.ResultSet\] messages, then
///     statement `N+1` in the request failed.
///
/// Example 1:
///
/// * Request: 5 DML statements, all executed successfully.
/// * Response: 5 \[ResultSet][google.spanner.v1.ResultSet\] messages, with the status `OK`.
///
/// Example 2:
///
/// * Request: 5 DML statements. The third statement has a syntax error.
/// * Response: 2 \[ResultSet][google.spanner.v1.ResultSet\] messages, and a syntax error (`INVALID_ARGUMENT`)
///    status. The number of \[ResultSet][google.spanner.v1.ResultSet\] messages indicates that the third
///    statement failed, and the fourth and fifth statements were not executed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecuteBatchDmlResponse {
    /// One \[ResultSet][google.spanner.v1.ResultSet\] for each statement in the request that ran successfully,
    /// in the same order as the statements in the request. Each \[ResultSet][google.spanner.v1.ResultSet\] does
    /// not contain any rows. The \[ResultSetStats][google.spanner.v1.ResultSetStats\] in each \[ResultSet][google.spanner.v1.ResultSet\] contain
    /// the number of rows modified by the statement.
    ///
    /// Only the first \[ResultSet][google.spanner.v1.ResultSet\] in the response contains valid
    /// \[ResultSetMetadata][google.spanner.v1.ResultSetMetadata\].
    #[prost(message, repeated, tag = "1")]
    pub result_sets: ::prost::alloc::vec::Vec<ResultSet>,
    /// If all DML statements are executed successfully, the status is `OK`.
    /// Otherwise, the error status of the first failed statement.
    #[prost(message, optional, tag = "2")]
    pub status: ::core::option::Option<super::super::rpc::Status>,
}
/// Options for a PartitionQueryRequest and
/// PartitionReadRequest.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartitionOptions {
    /// **Note:** This hint is currently ignored by PartitionQuery and
    /// PartitionRead requests.
    ///
    /// The desired data size for each partition generated.  The default for this
    /// option is currently 1 GiB.  This is only a hint. The actual size of each
    /// partition may be smaller or larger than this size request.
    #[prost(int64, tag = "1")]
    pub partition_size_bytes: i64,
    /// **Note:** This hint is currently ignored by PartitionQuery and
    /// PartitionRead requests.
    ///
    /// The desired maximum number of partitions to return.  For example, this may
    /// be set to the number of workers available.  The default for this option
    /// is currently 10,000. The maximum value is currently 200,000.  This is only
    /// a hint.  The actual number of partitions returned may be smaller or larger
    /// than this maximum count request.
    #[prost(int64, tag = "2")]
    pub max_partitions: i64,
}
/// The request for \[PartitionQuery][google.spanner.v1.Spanner.PartitionQuery\]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartitionQueryRequest {
    /// Required. The session used to create the partitions.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// Read only snapshot transactions are supported, read/write and single use
    /// transactions are not.
    #[prost(message, optional, tag = "2")]
    pub transaction: ::core::option::Option<TransactionSelector>,
    /// Required. The query request to generate partitions for. The request will fail if
    /// the query is not root partitionable. The query plan of a root
    /// partitionable query has a single distributed union operator. A distributed
    /// union operator conceptually divides one or more tables into multiple
    /// splits, remotely evaluates a subquery independently on each split, and
    /// then unions all results.
    ///
    /// This must not contain DML commands, such as INSERT, UPDATE, or
    /// DELETE. Use \[ExecuteStreamingSql][google.spanner.v1.Spanner.ExecuteStreamingSql\] with a
    /// PartitionedDml transaction for large, partition-friendly DML operations.
    #[prost(string, tag = "3")]
    pub sql: ::prost::alloc::string::String,
    /// Parameter names and values that bind to placeholders in the SQL string.
    ///
    /// A parameter placeholder consists of the `@` character followed by the
    /// parameter name (for example, `@firstName`). Parameter names can contain
    /// letters, numbers, and underscores.
    ///
    /// Parameters can appear anywhere that a literal value is expected.  The same
    /// parameter name can be used more than once, for example:
    ///
    /// `"WHERE id > @msg_id AND id < @msg_id + 100"`
    ///
    /// It is an error to execute a SQL statement with unbound parameters.
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<::prost_types::Struct>,
    /// It is not always possible for Cloud Spanner to infer the right SQL type
    /// from a JSON value.  For example, values of type `BYTES` and values
    /// of type `STRING` both appear in \[params][google.spanner.v1.PartitionQueryRequest.params\] as JSON strings.
    ///
    /// In these cases, `param_types` can be used to specify the exact
    /// SQL type for some or all of the SQL query parameters. See the
    /// definition of \[Type][google.spanner.v1.Type\] for more information
    /// about SQL types.
    #[prost(map = "string, message", tag = "5")]
    pub param_types: ::std::collections::HashMap<::prost::alloc::string::String, Type>,
    /// Additional options that affect how many partitions are created.
    #[prost(message, optional, tag = "6")]
    pub partition_options: ::core::option::Option<PartitionOptions>,
}
/// The request for \[PartitionRead][google.spanner.v1.Spanner.PartitionRead\]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartitionReadRequest {
    /// Required. The session used to create the partitions.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// Read only snapshot transactions are supported, read/write and single use
    /// transactions are not.
    #[prost(message, optional, tag = "2")]
    pub transaction: ::core::option::Option<TransactionSelector>,
    /// Required. The name of the table in the database to be read.
    #[prost(string, tag = "3")]
    pub table: ::prost::alloc::string::String,
    /// If non-empty, the name of an index on \[table][google.spanner.v1.PartitionReadRequest.table\]. This index is
    /// used instead of the table primary key when interpreting \[key_set][google.spanner.v1.PartitionReadRequest.key_set\]
    /// and sorting result rows. See \[key_set][google.spanner.v1.PartitionReadRequest.key_set\] for further information.
    #[prost(string, tag = "4")]
    pub index: ::prost::alloc::string::String,
    /// The columns of \[table][google.spanner.v1.PartitionReadRequest.table\] to be returned for each row matching
    /// this request.
    #[prost(string, repeated, tag = "5")]
    pub columns: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Required. `key_set` identifies the rows to be yielded. `key_set` names the
    /// primary keys of the rows in \[table][google.spanner.v1.PartitionReadRequest.table\] to be yielded, unless \[index][google.spanner.v1.PartitionReadRequest.index\]
    /// is present. If \[index][google.spanner.v1.PartitionReadRequest.index\] is present, then \[key_set][google.spanner.v1.PartitionReadRequest.key_set\] instead names
    /// index keys in \[index][google.spanner.v1.PartitionReadRequest.index\].
    ///
    /// It is not an error for the `key_set` to name rows that do not
    /// exist in the database. Read yields nothing for nonexistent rows.
    #[prost(message, optional, tag = "6")]
    pub key_set: ::core::option::Option<KeySet>,
    /// Additional options that affect how many partitions are created.
    #[prost(message, optional, tag = "9")]
    pub partition_options: ::core::option::Option<PartitionOptions>,
}
/// Information returned for each partition returned in a
/// PartitionResponse.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Partition {
    /// This token can be passed to Read, StreamingRead, ExecuteSql, or
    /// ExecuteStreamingSql requests to restrict the results to those identified by
    /// this partition token.
    #[prost(bytes = "bytes", tag = "1")]
    pub partition_token: ::prost::bytes::Bytes,
}
/// The response for \[PartitionQuery][google.spanner.v1.Spanner.PartitionQuery\]
/// or \[PartitionRead][google.spanner.v1.Spanner.PartitionRead\]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartitionResponse {
    /// Partitions created by this request.
    #[prost(message, repeated, tag = "1")]
    pub partitions: ::prost::alloc::vec::Vec<Partition>,
    /// Transaction created by this request.
    #[prost(message, optional, tag = "2")]
    pub transaction: ::core::option::Option<Transaction>,
}
/// The request for \[Read][google.spanner.v1.Spanner.Read\] and
/// \[StreamingRead][google.spanner.v1.Spanner.StreamingRead\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadRequest {
    /// Required. The session in which the read should be performed.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// The transaction to use. If none is provided, the default is a
    /// temporary read-only transaction with strong concurrency.
    #[prost(message, optional, tag = "2")]
    pub transaction: ::core::option::Option<TransactionSelector>,
    /// Required. The name of the table in the database to be read.
    #[prost(string, tag = "3")]
    pub table: ::prost::alloc::string::String,
    /// If non-empty, the name of an index on \[table][google.spanner.v1.ReadRequest.table\]. This index is
    /// used instead of the table primary key when interpreting \[key_set][google.spanner.v1.ReadRequest.key_set\]
    /// and sorting result rows. See \[key_set][google.spanner.v1.ReadRequest.key_set\] for further information.
    #[prost(string, tag = "4")]
    pub index: ::prost::alloc::string::String,
    /// Required. The columns of \[table][google.spanner.v1.ReadRequest.table\] to be returned for each row matching
    /// this request.
    #[prost(string, repeated, tag = "5")]
    pub columns: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Required. `key_set` identifies the rows to be yielded. `key_set` names the
    /// primary keys of the rows in \[table][google.spanner.v1.ReadRequest.table\] to be yielded, unless \[index][google.spanner.v1.ReadRequest.index\]
    /// is present. If \[index][google.spanner.v1.ReadRequest.index\] is present, then \[key_set][google.spanner.v1.ReadRequest.key_set\] instead names
    /// index keys in \[index][google.spanner.v1.ReadRequest.index\].
    ///
    /// If the \[partition_token][google.spanner.v1.ReadRequest.partition_token\] field is empty, rows are yielded
    /// in table primary key order (if \[index][google.spanner.v1.ReadRequest.index\] is empty) or index key order
    /// (if \[index][google.spanner.v1.ReadRequest.index\] is non-empty).  If the \[partition_token][google.spanner.v1.ReadRequest.partition_token\] field is not
    /// empty, rows will be yielded in an unspecified order.
    ///
    /// It is not an error for the `key_set` to name rows that do not
    /// exist in the database. Read yields nothing for nonexistent rows.
    #[prost(message, optional, tag = "6")]
    pub key_set: ::core::option::Option<KeySet>,
    /// If greater than zero, only the first `limit` rows are yielded. If `limit`
    /// is zero, the default is no limit. A limit cannot be specified if
    /// `partition_token` is set.
    #[prost(int64, tag = "8")]
    pub limit: i64,
    /// If this request is resuming a previously interrupted read,
    /// `resume_token` should be copied from the last
    /// \[PartialResultSet][google.spanner.v1.PartialResultSet\] yielded before the interruption. Doing this
    /// enables the new read to resume where the last read left off. The
    /// rest of the request parameters must exactly match the request
    /// that yielded this token.
    #[prost(bytes = "bytes", tag = "9")]
    pub resume_token: ::prost::bytes::Bytes,
    /// If present, results will be restricted to the specified partition
    /// previously created using PartitionRead().    There must be an exact
    /// match for the values of fields common to this message and the
    /// PartitionReadRequest message used to create this partition_token.
    #[prost(bytes = "bytes", tag = "10")]
    pub partition_token: ::prost::bytes::Bytes,
    /// Common options for this request.
    #[prost(message, optional, tag = "11")]
    pub request_options: ::core::option::Option<RequestOptions>,
    /// If this is for a partitioned read and this field is set to `true`, the
    /// request will be executed via Spanner independent compute resources.
    ///
    /// If the field is set to `true` but the request does not set
    /// `partition_token`, the API will return an `INVALID_ARGUMENT` error.
    #[prost(bool, tag = "15")]
    pub data_boost_enabled: bool,
}
/// The request for \[BeginTransaction][google.spanner.v1.Spanner.BeginTransaction\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BeginTransactionRequest {
    /// Required. The session in which the transaction runs.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// Required. Options for the new transaction.
    #[prost(message, optional, tag = "2")]
    pub options: ::core::option::Option<TransactionOptions>,
    /// Common options for this request.
    /// Priority is ignored for this request. Setting the priority in this
    /// request_options struct will not do anything. To set the priority for a
    /// transaction, set it on the reads and writes that are part of this
    /// transaction instead.
    #[prost(message, optional, tag = "3")]
    pub request_options: ::core::option::Option<RequestOptions>,
}
/// The request for \[Commit][google.spanner.v1.Spanner.Commit\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitRequest {
    /// Required. The session in which the transaction to be committed is running.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// The mutations to be executed when this transaction commits. All
    /// mutations are applied atomically, in the order they appear in
    /// this list.
    #[prost(message, repeated, tag = "4")]
    pub mutations: ::prost::alloc::vec::Vec<Mutation>,
    /// If `true`, then statistics related to the transaction will be included in
    /// the \[CommitResponse][google.spanner.v1.CommitResponse.commit_stats\]. Default value is
    /// `false`.
    #[prost(bool, tag = "5")]
    pub return_commit_stats: bool,
    /// Common options for this request.
    #[prost(message, optional, tag = "6")]
    pub request_options: ::core::option::Option<RequestOptions>,
    /// Required. The transaction in which to commit.
    #[prost(oneof = "commit_request::Transaction", tags = "2, 3")]
    pub transaction: ::core::option::Option<commit_request::Transaction>,
}
/// Nested message and enum types in `CommitRequest`.
pub mod commit_request {
    /// Required. The transaction in which to commit.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Transaction {
        /// Commit a previously-started transaction.
        #[prost(bytes, tag = "2")]
        TransactionId(::prost::bytes::Bytes),
        /// Execute mutations in a temporary transaction. Note that unlike
        /// commit of a previously-started transaction, commit with a
        /// temporary transaction is non-idempotent. That is, if the
        /// `CommitRequest` is sent to Cloud Spanner more than once (for
        /// instance, due to retries in the application, or in the
        /// transport library), it is possible that the mutations are
        /// executed more than once. If this is undesirable, use
        /// \[BeginTransaction][google.spanner.v1.Spanner.BeginTransaction\] and
        /// \[Commit][google.spanner.v1.Spanner.Commit\] instead.
        #[prost(message, tag = "3")]
        SingleUseTransaction(super::TransactionOptions),
    }
}
/// The request for \[Rollback][google.spanner.v1.Spanner.Rollback\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RollbackRequest {
    /// Required. The session in which the transaction to roll back is running.
    #[prost(string, tag = "1")]
    pub session: ::prost::alloc::string::String,
    /// Required. The transaction to roll back.
    #[prost(bytes = "bytes", tag = "2")]
    pub transaction_id: ::prost::bytes::Bytes,
}
/// Generated client implementations.
pub mod spanner_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Cloud Spanner API
    ///
    /// The Cloud Spanner API can be used to manage sessions and execute
    /// transactions on data stored in Cloud Spanner databases.
    #[derive(Debug, Clone)]
    pub struct SpannerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SpannerClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> SpannerClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> SpannerClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            SpannerClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Creates a new session. A session can be used to perform
        /// transactions that read and/or modify data in a Cloud Spanner database.
        /// Sessions are meant to be reused for many consecutive
        /// transactions.
        ///
        /// Sessions can only execute one transaction at a time. To execute
        /// multiple concurrent read-write/write-only transactions, create
        /// multiple sessions. Note that standalone reads and queries use a
        /// transaction internally, and count toward the one transaction
        /// limit.
        ///
        /// Active sessions use additional server resources, so it is a good idea to
        /// delete idle and unneeded sessions.
        /// Aside from explicit deletes, Cloud Spanner may delete sessions for which no
        /// operations are sent for more than an hour. If a session is deleted,
        /// requests to it return `NOT_FOUND`.
        ///
        /// Idle sessions can be kept alive by sending a trivial SQL query
        /// periodically, e.g., `"SELECT 1"`.
        pub async fn create_session(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSessionRequest>,
        ) -> Result<tonic::Response<super::Session>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/CreateSession",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates multiple new sessions.
        ///
        /// This API can be used to initialize a session cache on the clients.
        /// See https://goo.gl/TgSFN2 for best practices on session cache management.
        pub async fn batch_create_sessions(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchCreateSessionsRequest>,
        ) -> Result<tonic::Response<super::BatchCreateSessionsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/BatchCreateSessions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets a session. Returns `NOT_FOUND` if the session does not exist.
        /// This is mainly useful for determining whether a session is still
        /// alive.
        pub async fn get_session(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSessionRequest>,
        ) -> Result<tonic::Response<super::Session>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/GetSession",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists all sessions in a given database.
        pub async fn list_sessions(
            &mut self,
            request: impl tonic::IntoRequest<super::ListSessionsRequest>,
        ) -> Result<tonic::Response<super::ListSessionsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/ListSessions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Ends a session, releasing server resources associated with it. This will
        /// asynchronously trigger cancellation of any operations that are running with
        /// this session.
        pub async fn delete_session(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteSessionRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/DeleteSession",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Executes an SQL statement, returning all results in a single reply. This
        /// method cannot be used to return a result set larger than 10 MiB;
        /// if the query yields more data than that, the query fails with
        /// a `FAILED_PRECONDITION` error.
        ///
        /// Operations inside read-write transactions might return `ABORTED`. If
        /// this occurs, the application should restart the transaction from
        /// the beginning. See [Transaction][google.spanner.v1.Transaction] for more details.
        ///
        /// Larger result sets can be fetched in streaming fashion by calling
        /// [ExecuteStreamingSql][google.spanner.v1.Spanner.ExecuteStreamingSql] instead.
        pub async fn execute_sql(
            &mut self,
            request: impl tonic::IntoRequest<super::ExecuteSqlRequest>,
        ) -> Result<tonic::Response<super::ResultSet>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/ExecuteSql",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Like [ExecuteSql][google.spanner.v1.Spanner.ExecuteSql], except returns the result
        /// set as a stream. Unlike [ExecuteSql][google.spanner.v1.Spanner.ExecuteSql], there
        /// is no limit on the size of the returned result set. However, no
        /// individual row in the result set can exceed 100 MiB, and no
        /// column value can exceed 10 MiB.
        pub async fn execute_streaming_sql(
            &mut self,
            request: impl tonic::IntoRequest<super::ExecuteSqlRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::PartialResultSet>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/ExecuteStreamingSql",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Executes a batch of SQL DML statements. This method allows many statements
        /// to be run with lower latency than submitting them sequentially with
        /// [ExecuteSql][google.spanner.v1.Spanner.ExecuteSql].
        ///
        /// Statements are executed in sequential order. A request can succeed even if
        /// a statement fails. The [ExecuteBatchDmlResponse.status][google.spanner.v1.ExecuteBatchDmlResponse.status] field in the
        /// response provides information about the statement that failed. Clients must
        /// inspect this field to determine whether an error occurred.
        ///
        /// Execution stops after the first failed statement; the remaining statements
        /// are not executed.
        pub async fn execute_batch_dml(
            &mut self,
            request: impl tonic::IntoRequest<super::ExecuteBatchDmlRequest>,
        ) -> Result<tonic::Response<super::ExecuteBatchDmlResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/ExecuteBatchDml",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Reads rows from the database using key lookups and scans, as a
        /// simple key/value style alternative to
        /// [ExecuteSql][google.spanner.v1.Spanner.ExecuteSql].  This method cannot be used to
        /// return a result set larger than 10 MiB; if the read matches more
        /// data than that, the read fails with a `FAILED_PRECONDITION`
        /// error.
        ///
        /// Reads inside read-write transactions might return `ABORTED`. If
        /// this occurs, the application should restart the transaction from
        /// the beginning. See [Transaction][google.spanner.v1.Transaction] for more details.
        ///
        /// Larger result sets can be yielded in streaming fashion by calling
        /// [StreamingRead][google.spanner.v1.Spanner.StreamingRead] instead.
        pub async fn read(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadRequest>,
        ) -> Result<tonic::Response<super::ResultSet>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/Read",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Like [Read][google.spanner.v1.Spanner.Read], except returns the result set as a
        /// stream. Unlike [Read][google.spanner.v1.Spanner.Read], there is no limit on the
        /// size of the returned result set. However, no individual row in
        /// the result set can exceed 100 MiB, and no column value can exceed
        /// 10 MiB.
        pub async fn streaming_read(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::PartialResultSet>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/StreamingRead",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Begins a new transaction. This step can often be skipped:
        /// [Read][google.spanner.v1.Spanner.Read], [ExecuteSql][google.spanner.v1.Spanner.ExecuteSql] and
        /// [Commit][google.spanner.v1.Spanner.Commit] can begin a new transaction as a
        /// side-effect.
        pub async fn begin_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::BeginTransactionRequest>,
        ) -> Result<tonic::Response<super::Transaction>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/BeginTransaction",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Commits a transaction. The request includes the mutations to be
        /// applied to rows in the database.
        ///
        /// `Commit` might return an `ABORTED` error. This can occur at any time;
        /// commonly, the cause is conflicts with concurrent
        /// transactions. However, it can also happen for a variety of other
        /// reasons. If `Commit` returns `ABORTED`, the caller should re-attempt
        /// the transaction from the beginning, re-using the same session.
        ///
        /// On very rare occasions, `Commit` might return `UNKNOWN`. This can happen,
        /// for example, if the client job experiences a 1+ hour networking failure.
        /// At that point, Cloud Spanner has lost track of the transaction outcome and
        /// we recommend that you perform another read from the database to see the
        /// state of things as they are now.
        pub async fn commit(
            &mut self,
            request: impl tonic::IntoRequest<super::CommitRequest>,
        ) -> Result<tonic::Response<super::CommitResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/Commit",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Rolls back a transaction, releasing any locks it holds. It is a good
        /// idea to call this for any transaction that includes one or more
        /// [Read][google.spanner.v1.Spanner.Read] or [ExecuteSql][google.spanner.v1.Spanner.ExecuteSql] requests and
        /// ultimately decides not to commit.
        ///
        /// `Rollback` returns `OK` if it successfully aborts the transaction, the
        /// transaction was already aborted, or the transaction is not
        /// found. `Rollback` never returns `ABORTED`.
        pub async fn rollback(
            &mut self,
            request: impl tonic::IntoRequest<super::RollbackRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/Rollback",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates a set of partition tokens that can be used to execute a query
        /// operation in parallel.  Each of the returned partition tokens can be used
        /// by [ExecuteStreamingSql][google.spanner.v1.Spanner.ExecuteStreamingSql] to specify a subset
        /// of the query result to read.  The same session and read-only transaction
        /// must be used by the PartitionQueryRequest used to create the
        /// partition tokens and the ExecuteSqlRequests that use the partition tokens.
        ///
        /// Partition tokens become invalid when the session used to create them
        /// is deleted, is idle for too long, begins a new transaction, or becomes too
        /// old.  When any of these happen, it is not possible to resume the query, and
        /// the whole operation must be restarted from the beginning.
        pub async fn partition_query(
            &mut self,
            request: impl tonic::IntoRequest<super::PartitionQueryRequest>,
        ) -> Result<tonic::Response<super::PartitionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/PartitionQuery",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates a set of partition tokens that can be used to execute a read
        /// operation in parallel.  Each of the returned partition tokens can be used
        /// by [StreamingRead][google.spanner.v1.Spanner.StreamingRead] to specify a subset of the read
        /// result to read.  The same session and read-only transaction must be used by
        /// the PartitionReadRequest used to create the partition tokens and the
        /// ReadRequests that use the partition tokens.  There are no ordering
        /// guarantees on rows returned among the returned partition tokens, or even
        /// within each individual StreamingRead call issued with a partition_token.
        ///
        /// Partition tokens become invalid when the session used to create them
        /// is deleted, is idle for too long, begins a new transaction, or becomes too
        /// old.  When any of these happen, it is not possible to resume the read, and
        /// the whole operation must be restarted from the beginning.
        pub async fn partition_read(
            &mut self,
            request: impl tonic::IntoRequest<super::PartitionReadRequest>,
        ) -> Result<tonic::Response<super::PartitionResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.v1.Spanner/PartitionRead",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}

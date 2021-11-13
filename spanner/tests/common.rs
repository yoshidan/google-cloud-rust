use anyhow::Result;
use chrono::{NaiveDate, Utc, DateTime};
use google_cloud_googleapis::spanner::v1::commit_request::Transaction::SingleUseTransaction;
use google_cloud_googleapis::spanner::v1::transaction_options::{Mode, ReadWrite};
use google_cloud_googleapis::spanner::v1::{
    CommitRequest, CommitResponse, Mutation, TransactionOptions,
};
use google_cloud_spanner::apiv1::conn_pool::ConnectionManager;
use google_cloud_spanner::key::{Key, KeySet};
use google_cloud_spanner::mutation::insert_or_update;
use google_cloud_spanner::reader::{AsyncIterator, RowIterator};
use google_cloud_spanner::row::{Row, Struct, TryFromStruct};
use google_cloud_spanner::sessions::{
    ManagedSession, SessionConfig, SessionHandle, SessionManager,
};
use google_cloud_spanner::statement::{Statement, ToKind};
use google_cloud_spanner::transaction::CallOptions;
use google_cloud_spanner::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use google_cloud_spanner::value::{CommitTimestamp, TimestampBound};
use rust_decimal::Decimal;
use std::str::FromStr;

pub const DATABASE: &str =
    "projects/local-project/instances/test-instance/databases/local-database";

pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
    pub level: i64,
    pub updated_at: CommitTimestamp,
}

impl TryFromStruct for UserCharacter {
    fn try_from(s: Struct<'_>) -> Result<Self> {
        Ok(UserCharacter {
            user_id: s.column_by_name("UserId")?,
            character_id: s.column_by_name("CharacterId")?,
            level: s.column_by_name("Level")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}

pub struct UserItem {
    pub user_id: String,
    pub item_id: i64,
    pub quantity: i64,
    pub updated_at: CommitTimestamp,
}

impl TryFromStruct for UserItem {
    fn try_from(s: Struct<'_>) -> Result<Self> {
        Ok(UserItem {
            user_id: s.column_by_name("UserId")?,
            item_id: s.column_by_name("ItemId")?,
            quantity: s.column_by_name("Quantity")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}

pub fn user_columns() -> Vec<&'static str> {
    vec![
        "UserId",
        "NotNullINT64",
        "NullableINT64",
        "NotNullFloat64",
        "NullableFloat64",
        "NotNullBool",
        "NullableBool",
        "NotNullByteArray",
        "NullableByteArray",
        "NotNullNumeric",
        "NullableNumeric",
        "NotNullTimestamp",
        "NullableTimestamp",
        "NotNullDate",
        "NullableDate",
        "NotNullArray",
        "NullableArray",
        "NullableString",
        "UpdatedAt",
    ]
}

pub async fn create_session() -> ManagedSession {
    let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
        .await
        .unwrap();
    let mut config = SessionConfig::default();
    config.min_opened = 1;
    config.max_opened = 1;
    SessionManager::new(DATABASE, cm, config)
        .await
        .unwrap()
        .get()
        .await
        .unwrap()
}

pub async fn replace_test_data(
    session: &mut SessionHandle,
    mutations: Vec<Mutation>,
) -> Result<CommitResponse, tonic::Status> {
    session
        .spanner_client
        .commit(
            CommitRequest {
                session: session.session.name.to_string(),
                mutations,
                return_commit_stats: false,
                request_options: None,
                transaction: Some(SingleUseTransaction(TransactionOptions {
                    mode: Some(Mode::ReadWrite(ReadWrite {})),
                })),
            },
            None,
        )
        .await
        .map(|x| x.into_inner())
}

pub fn create_user_mutation(user_id: &str, now: &DateTime<Utc>) -> Mutation {
    insert_or_update(
        "User",
        user_columns(),
        vec![
            user_id.to_kind(),
            1.to_kind(),
            None::<i64>.to_kind(),
            1.0.to_kind(),
            None::<f64>.to_kind(),
            true.to_kind(),
            None::<bool>.to_kind(),
            vec![1_u8].to_kind(),
            None::<Vec<u8>>.to_kind(),
            Decimal::from_str("100.24").unwrap().to_kind(),
            Some(Decimal::from_str("1000.42342").unwrap()).to_kind(),
            now.to_kind(),
            Some(*now).to_kind(),
            now.naive_utc().date().to_kind(),
            None::<DateTime<Utc>>.to_kind(),
            vec![10_i64, 20_i64, 30_i64].to_kind(),
            None::<Vec<i64>>.to_kind(),
            Some(user_id).to_kind(),
            CommitTimestamp::new().to_kind(),
        ],
    )
}

pub fn create_user_item_mutation(user_id: &str, item_id: i64) -> Mutation {
    insert_or_update(
        "UserItem",
        vec!["UserId", "ItemId", "Quantity", "UpdatedAt"],
        vec![
            user_id.to_kind(),
            item_id.to_kind(),
            100.to_kind(),
            CommitTimestamp::new().to_kind(),
        ],
    )
}

pub fn create_user_character_mutation(user_id: &str, character_id: i64) -> Mutation {
    insert_or_update(
        "UserCharacter",
        vec!["UserId", "CharacterId", "Level", "UpdatedAt"],
        vec![
            user_id.to_kind(),
            character_id.to_kind(),
            1.to_kind(),
            CommitTimestamp::new().to_kind(),
        ],
    )
}

pub fn assert_user_row(
    row: &Row,
    source_user_id: &str,
    now: &DateTime<Utc>,
    commit_timestamp: &DateTime<Utc>,
) {
    let user_id = row.column_by_name::<String>("UserId").unwrap();
    assert_eq!(user_id, source_user_id);
    let not_null_int64 = row.column_by_name::<i64>("NotNullINT64").unwrap();
    assert_eq!(not_null_int64, 1);
    let nullable_int64 = row.column_by_name::<Option<i64>>("NullableINT64").unwrap();
    assert_eq!(nullable_int64, None);
    let not_null_float64 = row.column_by_name::<f64>("NotNullFloat64").unwrap();
    assert_eq!(not_null_float64, 1.0);
    let nullable_float64 = row
        .column_by_name::<Option<f64>>("NullableFloat64")
        .unwrap();
    assert_eq!(nullable_float64, None);
    let not_null_bool = row.column_by_name::<bool>("NotNullBool").unwrap();
    assert_eq!(not_null_bool, true);
    let nullable_bool = row.column_by_name::<Option<bool>>("NullableBool").unwrap();
    assert_eq!(nullable_bool, None);
    let mut not_null_byte_array = row.column_by_name::<Vec<u8>>("NotNullByteArray").unwrap();
    assert_eq!(not_null_byte_array.pop().unwrap(), 1_u8);
    let nullable_byte_array = row
        .column_by_name::<Option<Vec<u8>>>("NullableByteArray")
        .unwrap();
    assert_eq!(nullable_byte_array, None);
    let not_null_decimal = row.column_by_name::<Decimal>("NotNullNumeric").unwrap();
    assert_eq!(not_null_decimal.to_string(), "100.24");
    let nullable_decimal = row
        .column_by_name::<Option<Decimal>>("NullableNumeric")
        .unwrap();
    assert_eq!(nullable_decimal.unwrap().to_string(), "1000.42342");
    let not_null_ts = row
        .column_by_name::<DateTime<Utc>>("NotNullTimestamp")
        .unwrap();
    assert_eq!(not_null_ts.to_string(), now.to_string());
    let nullable_ts = row
        .column_by_name::<Option<DateTime<Utc>>>("NullableTimestamp")
        .unwrap();
    assert_eq!(nullable_ts.unwrap().to_string(), now.to_string());
    let not_null_date = row.column_by_name::<NaiveDate>("NotNullDate").unwrap();
    assert_eq!(not_null_date.to_string(), now.naive_utc().date().to_string());
    let nullable_date = row
        .column_by_name::<Option<NaiveDate>>("NullableDate")
        .unwrap();
    assert_eq!(nullable_date, None);
    let mut not_null_array = row.column_by_name::<Vec<i64>>("NotNullArray").unwrap();
    assert_eq!(not_null_array.pop().unwrap(), 30); // from tail
    assert_eq!(not_null_array.pop().unwrap(), 20);
    assert_eq!(not_null_array.pop().unwrap(), 10);
    let nullable_array = row
        .column_by_name::<Option<Vec<i64>>>("NullableArray")
        .unwrap();
    assert_eq!(nullable_array, None);
    let nullable_string = row
        .column_by_name::<Option<String>>("NullableString")
        .unwrap();
    assert_eq!(nullable_string.unwrap(), user_id);
    let updated_at = row.column_by_name::<CommitTimestamp>("UpdatedAt").unwrap();
    assert_eq!(
        updated_at.timestamp.to_string(),
        commit_timestamp.to_string(),
        "commit timestamp"
    );
}

pub async fn read_only_transaction(session: ManagedSession) -> ReadOnlyTransaction {
    match ReadOnlyTransaction::begin(
        session,
        TimestampBound::strong_read(),
        CallOptions::default(),
    )
    .await
    {
        Ok(tx) => tx,
        Err(status) => panic!("begin error {:?}", status),
    }
}

pub async fn all_rows(mut itr: RowIterator<'_>) -> Vec<Row> {
    let mut rows = vec![];
    loop {
        match itr.next().await {
            Ok(row) => {
                if row.is_some() {
                    rows.push(row.unwrap());
                } else {
                    break;
                }
            }
            Err(status) => panic!("reader aborted {:?}", status),
        };
    }
    rows
}

pub async fn assert_partitioned_query(
    tx: &mut BatchReadOnlyTransaction,
    user_id: &str,
    now: &DateTime<Utc>,
    cts: &DateTime<Utc>,
) {
    let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
    stmt.add_param("UserId", user_id);
    let row = execute_partitioned_query(tx, stmt).await;
    assert_eq!(row.len(), 1);
    assert_user_row(row.first().unwrap(), user_id, now, cts);
}

pub async fn execute_partitioned_query(
    tx: &mut BatchReadOnlyTransaction,
    stmt: Statement,
) -> Vec<Row> {
    let partitions = match tx.partition_query(stmt).await {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    println!("partition count = {}", partitions.len());
    let mut rows = vec![];
    for p in partitions.into_iter() {
        let reader = match tx.execute(p).await {
            Ok(tx) => tx,
            Err(status) => panic!("query error {:?}", status),
        };
        let rows_per_partition = all_rows(reader).await;
        for x in rows_per_partition {
            rows.push(x);
        }
    }
    rows
}

pub async fn assert_partitioned_read(
    tx: &mut BatchReadOnlyTransaction,
    user_id: &str,
    now: &DateTime<Utc>,
    cts: &DateTime<Utc>,
) {
    let partitions = match tx
        .partition_read("User", user_columns(), KeySet::from(Key::one(user_id)))
        .await
    {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    println!("partition count = {}", partitions.len());
    let mut rows = vec![];
    for p in partitions.into_iter() {
        let reader = match tx.execute(p).await {
            Ok(tx) => tx,
            Err(status) => panic!("query error {:?}", status),
        };
        let rows_per_partition = all_rows(reader).await;
        for x in rows_per_partition {
            rows.push(x);
        }
    }
    assert_eq!(rows.len(), 1);
    assert_user_row(rows.first().unwrap(), user_id, now, cts);
}

use anyhow::Result;
use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::Status;
use google_cloud_googleapis::spanner::v1::commit_request::Transaction::SingleUseTransaction;
use google_cloud_googleapis::spanner::v1::transaction_options::{Mode, ReadWrite};
use google_cloud_googleapis::spanner::v1::{CommitRequest, CommitResponse, Mutation, TransactionOptions};
use google_cloud_spanner::apiv1::conn_pool::ConnectionManager;
use google_cloud_spanner::key::Key;
use google_cloud_spanner::mutation::insert_or_update;
use google_cloud_spanner::reader::{AsyncIterator, RowIterator};
use google_cloud_spanner::row::{Error as RowError, Row, Struct, TryFromStruct};
use google_cloud_spanner::session::{ManagedSession, SessionConfig, SessionHandle, SessionManager};
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::CallOptions;
use google_cloud_spanner::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use google_cloud_spanner::value::{CommitTimestamp, SpannerNumeric, TimestampBound};
use time::{Date, OffsetDateTime};

pub const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
    pub level: i64,
    pub updated_at: CommitTimestamp,
}

impl TryFromStruct for UserCharacter {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, RowError> {
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
    fn try_from_struct(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(UserItem {
            user_id: s.column_by_name("UserId")?,
            item_id: s.column_by_name("ItemId")?,
            quantity: s.column_by_name("Quantity")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub async fn create_session() -> ManagedSession {
    let cm = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
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

#[allow(dead_code)]
pub async fn replace_test_data(
    session: &mut SessionHandle,
    mutations: Vec<Mutation>,
) -> Result<CommitResponse, Status> {
    session
        .spanner_client
        .commit(
            CommitRequest {
                session: session.session.name.to_string(),
                mutations,
                return_commit_stats: false,
                request_options: None,
                transaction: Some(SingleUseTransaction(TransactionOptions {
                    mode: Some(Mode::ReadWrite(ReadWrite::default())),
                })),
            },
            None,
            None,
        )
        .await
        .map(|x| x.into_inner())
}

#[allow(dead_code)]
pub fn create_user_mutation(user_id: &str, now: &OffsetDateTime) -> Mutation {
    insert_or_update(
        "User",
        &user_columns(),
        &[
            &user_id,
            &1,
            &None::<i64>,
            &1.0,
            &None::<f64>,
            &true,
            &None::<bool>,
            &vec![1_u8],
            &None::<Vec<u8>>,
            &SpannerNumeric::new("100.24"),
            &Some(SpannerNumeric::new("1000.42342")),
            now,
            &Some(*now),
            &now.date(),
            &None::<OffsetDateTime>,
            &vec![10_i64, 20_i64, 30_i64],
            &None::<Vec<i64>>,
            &Some(user_id),
            &CommitTimestamp::new(),
        ],
    )
}

#[allow(dead_code)]
pub fn create_user_item_mutation(user_id: &str, item_id: i64) -> Mutation {
    insert_or_update(
        "UserItem",
        &["UserId", "ItemId", "Quantity", "UpdatedAt"],
        &[&user_id, &item_id, &100, &CommitTimestamp::new()],
    )
}

#[allow(dead_code)]
pub fn create_user_character_mutation(user_id: &str, character_id: i64) -> Mutation {
    insert_or_update(
        "UserCharacter",
        &["UserId", "CharacterId", "Level", "UpdatedAt"],
        &[&user_id, &character_id, &1, &CommitTimestamp::new()],
    )
}

#[allow(dead_code)]
pub fn assert_user_row(row: &Row, source_user_id: &str, now: &OffsetDateTime, commit_timestamp: &OffsetDateTime) {
    let user_id = row.column_by_name::<String>("UserId").unwrap();
    assert_eq!(user_id, source_user_id);
    let not_null_int64 = row.column_by_name::<i64>("NotNullINT64").unwrap();
    assert_eq!(not_null_int64, 1);
    let nullable_int64 = row.column_by_name::<Option<i64>>("NullableINT64").unwrap();
    assert_eq!(nullable_int64, None);
    let not_null_float64 = row.column_by_name::<f64>("NotNullFloat64").unwrap();
    assert_eq!(not_null_float64, 1.0);
    let nullable_float64 = row.column_by_name::<Option<f64>>("NullableFloat64").unwrap();
    assert_eq!(nullable_float64, None);
    let not_null_bool = row.column_by_name::<bool>("NotNullBool").unwrap();
    assert!(not_null_bool);
    let nullable_bool = row.column_by_name::<Option<bool>>("NullableBool").unwrap();
    assert_eq!(nullable_bool, None);
    let mut not_null_byte_array = row.column_by_name::<Vec<u8>>("NotNullByteArray").unwrap();
    assert_eq!(not_null_byte_array.pop().unwrap(), 1_u8);
    let nullable_byte_array = row.column_by_name::<Option<Vec<u8>>>("NullableByteArray").unwrap();
    assert_eq!(nullable_byte_array, None);
    let not_null_decimal = row.column_by_name::<SpannerNumeric>("NotNullNumeric").unwrap();
    assert_eq!(not_null_decimal.as_str(), "100.24");
    let nullable_decimal = row.column_by_name::<Option<SpannerNumeric>>("NullableNumeric").unwrap();
    assert_eq!(nullable_decimal.unwrap().as_str(), "1000.42342");
    let not_null_ts = row.column_by_name::<OffsetDateTime>("NotNullTimestamp").unwrap();
    assert_eq!(not_null_ts.to_string(), now.to_string());
    let nullable_ts = row
        .column_by_name::<Option<OffsetDateTime>>("NullableTimestamp")
        .unwrap();
    assert_eq!(nullable_ts.unwrap().to_string(), now.to_string());
    let not_null_date = row.column_by_name::<Date>("NotNullDate").unwrap();
    assert_eq!(not_null_date.to_string(), now.date().to_string());
    let nullable_date = row.column_by_name::<Option<Date>>("NullableDate").unwrap();
    assert_eq!(nullable_date, None);
    let mut not_null_array = row.column_by_name::<Vec<i64>>("NotNullArray").unwrap();
    assert_eq!(not_null_array.pop().unwrap(), 30); // from tail
    assert_eq!(not_null_array.pop().unwrap(), 20);
    assert_eq!(not_null_array.pop().unwrap(), 10);
    let nullable_array = row.column_by_name::<Option<Vec<i64>>>("NullableArray").unwrap();
    assert_eq!(nullable_array, None);
    let nullable_string = row.column_by_name::<Option<String>>("NullableString").unwrap();
    assert_eq!(nullable_string.unwrap(), user_id);
    let updated_at = row.column_by_name::<CommitTimestamp>("UpdatedAt").unwrap();
    assert_eq!(
        OffsetDateTime::from(updated_at).to_string(),
        commit_timestamp.to_string(),
        "commit timestamp"
    );
}

#[allow(dead_code)]
pub async fn read_only_transaction(session: ManagedSession) -> ReadOnlyTransaction {
    match ReadOnlyTransaction::begin(session, TimestampBound::strong_read(), CallOptions::default()).await {
        Ok(tx) => tx,
        Err(status) => panic!("begin error {:?}", status),
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub async fn assert_partitioned_query(
    tx: &mut BatchReadOnlyTransaction,
    user_id: &str,
    now: &OffsetDateTime,
    cts: &OffsetDateTime,
) {
    let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
    stmt.add_param("UserId", &user_id);
    let row = execute_partitioned_query(tx, stmt).await;
    assert_eq!(row.len(), 1);
    assert_user_row(row.first().unwrap(), user_id, now, cts);
}

#[allow(dead_code)]
pub async fn execute_partitioned_query(tx: &mut BatchReadOnlyTransaction, stmt: Statement) -> Vec<Row> {
    let partitions = match tx.partition_query(stmt).await {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    println!("partition count = {}", partitions.len());
    let mut rows = vec![];
    for p in partitions.into_iter() {
        let reader = match tx.execute(p, None).await {
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

#[allow(dead_code)]
pub async fn assert_partitioned_read(
    tx: &mut BatchReadOnlyTransaction,
    user_id: &str,
    now: &OffsetDateTime,
    cts: &OffsetDateTime,
) {
    let partitions = match tx
        .partition_read("User", &user_columns(), vec![Key::new(&user_id)])
        .await
    {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    println!("partition count = {}", partitions.len());
    let mut rows = vec![];
    for p in partitions.into_iter() {
        let reader = match tx.execute(p, None).await {
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

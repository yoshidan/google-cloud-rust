use anyhow::Result;
use chrono::NaiveDateTime;
use google_cloud_googleapis::spanner::v1::commit_request::Transaction::SingleUseTransaction;
use google_cloud_googleapis::spanner::v1::transaction_options::{Mode, ReadWrite};
use google_cloud_googleapis::spanner::v1::{
    CommitRequest, CommitResponse, Mutation, TransactionOptions,
};
use google_cloud_spanner::apiv1::conn_pool::ConnectionManager;
use google_cloud_spanner::mutation::insert_or_update;
use google_cloud_spanner::reader::{AsyncIterator, RowIterator};
use google_cloud_spanner::row::{Row, Struct, TryFromStruct};
use google_cloud_spanner::session_pool::{
    ManagedSession, SessionConfig, SessionHandle, SessionManager,
};
use google_cloud_spanner::statement::ToKind;
use google_cloud_spanner::value::CommitTimestamp;
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

pub fn create_user_mutation(user_id: &str, now: &NaiveDateTime) -> Mutation {
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
            vec![1 as u8].to_kind(),
            None::<Vec<u8>>.to_kind(),
            Decimal::from_str("100.24").unwrap().to_kind(),
            Some(Decimal::from_str("1000.42342").unwrap()).to_kind(),
            now.to_kind(),
            Some(now.clone()).to_kind(),
            now.date().to_kind(),
            None::<NaiveDateTime>.to_kind(),
            vec![10 as i64, 20 as i64, 30 as i64].to_kind(),
            None::<Vec<i64>>.to_kind(),
            Some(user_id).to_kind(),
            CommitTimestamp::new().to_kind(),
        ],
    )
}

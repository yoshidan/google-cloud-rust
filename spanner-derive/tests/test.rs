use chrono::{DateTime, NaiveDate, Utc};
use google_cloud_spanner::client::Client;
use google_cloud_spanner::mutation::insert_struct;
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::value::SpannerNumeric;
use google_cloud_spanner_derive::{Query, Table};
use serde::{Deserialize, Serialize};
use serial_test::serial;

#[derive(Table, Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
    pub level: i64,
    #[spanner(commitTimestamp)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Table, Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserItem {
    pub user_id: String,
    pub item_id: i64,
    pub quantity: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Table, Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub user_id: String,
    #[spanner(name = "NotNullINT64")]
    pub not_null_int64: i64,
    #[spanner(name = "NullableINT64")]
    pub nullable_int64: Option<i64>,
    pub not_null_float64: f64,
    pub nullable_float64: Option<f64>,
    pub not_null_bool: bool,
    pub nullable_bool: Option<bool>,
    pub not_null_byte_array: Vec<u8>,
    pub nullable_byte_array: Option<Vec<u8>>,
    pub not_null_numeric: SpannerNumeric,
    pub nullable_numeric: Option<SpannerNumeric>,
    pub not_null_timestamp: DateTime<Utc>,
    pub nullable_timestamp: Option<DateTime<Utc>>,
    pub not_null_date: NaiveDate,
    pub nullable_date: Option<NaiveDate>,
    pub not_null_array: Vec<i64>,
    pub nullable_array: Option<Vec<i64>>,
    pub nullable_string: Option<String>,
    #[spanner(commitTimestamp)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Query)]
pub struct UserBundle {
    pub user_id: String,
    pub user_characters: Vec<UserCharacter>,
    pub user_items: Vec<UserItem>,
}

#[tokio::test]
#[serial]
async fn test_table_derive() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let client = Client::new("projects/local-project/instances/test-instance/databases/local-database").await?;

    let now = Utc::now().timestamp();
    let user_id = format!("user{}", now);
    let user = User {
        user_id: user_id.clone(),
        not_null_numeric: SpannerNumeric::new("-99999999999999999999999999999.999999999"),
        ..Default::default()
    };
    client.apply(vec![insert_struct("User", user)]).await?;

    let mut tx = client.read_only_transaction().await?;
    let mut stmt = Statement::new("SELECT * From User WHERE UserID = @UserID");
    stmt.add_param("UserID", &user_id);
    let mut reader = tx.query(stmt).await?;
    if let Some(row) = reader.next().await? {
        let v: User = row.try_into()?;
        assert_eq!(v.user_id, user_id);
        assert_eq!(v.not_null_numeric.as_str(), "-99999999999999999999999999999.999999999");
        assert!(v.updated_at.timestamp() >= now);
        let json_string = serde_json::to_string(&v)?;
        let des = serde_json::from_str::<User>(json_string.as_str())?;
        assert_eq!(des, v);
    } else {
        panic!("no data found");
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_derive() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let client = Client::new("projects/local-project/instances/test-instance/databases/local-database").await?;

    let now = Utc::now().timestamp();
    let user_id = format!("user{}", now);
    let user = User {
        user_id: user_id.clone(),
        ..Default::default()
    };
    let user_character = UserCharacter {
        user_id: user_id.clone(),
        ..Default::default()
    };
    let user_item = UserItem {
        user_id: user_id.clone(),
        ..Default::default()
    };
    client
        .apply(vec![
            insert_struct("User", user),
            insert_struct("UserCharacter", user_character),
            insert_struct("UserItem", user_item),
        ])
        .await?;

    let mut tx = client.read_only_transaction().await?;
    let mut stmt = Statement::new(
        "
    SELECT
        UserId,
	    ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @UserId) AS UserCharacters,
	    ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = @UserId) AS UserItems,
    From User
    WHERE UserID = @UserID",
    );
    stmt.add_param("UserID", &user_id);
    let mut reader = tx.query(stmt).await?;
    if let Some(row) = reader.next().await? {
        let v: UserBundle = row.try_into()?;
        assert_eq!(v.user_id, user_id);
        assert_eq!(v.user_characters.len(), 1);
        assert_eq!(v.user_items.len(), 1);
    } else {
        panic!("no data found");
    }
    Ok(())
}

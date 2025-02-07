use google_cloud_spanner::bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::str::FromStr;
use time::{Date, OffsetDateTime};

use gcloud_spanner_derive as google_cloud_spanner_derive;
use google_cloud_spanner::client::{Client, ClientConfig, Error};
use google_cloud_spanner::mutation::insert_struct;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner_derive::{Query, Table};

#[derive(Table, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
    pub level: i64,
    #[spanner(commitTimestamp)]
    pub updated_at: OffsetDateTime,
}

impl Default for UserCharacter {
    fn default() -> Self {
        Self {
            updated_at: OffsetDateTime::UNIX_EPOCH,
            user_id: Default::default(),
            character_id: Default::default(),
            level: Default::default(),
        }
    }
}

#[derive(Table, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserItem {
    pub user_id: String,
    pub item_id: i64,
    pub quantity: i64,
    pub updated_at: OffsetDateTime,
}

impl Default for UserItem {
    fn default() -> Self {
        Self {
            updated_at: OffsetDateTime::UNIX_EPOCH,
            user_id: Default::default(),
            item_id: Default::default(),
            quantity: Default::default(),
        }
    }
}

#[derive(Table, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub not_null_numeric: BigDecimal,
    pub nullable_numeric: Option<BigDecimal>,
    pub not_null_timestamp: OffsetDateTime,
    pub nullable_timestamp: Option<OffsetDateTime>,
    pub not_null_date: Date,
    pub nullable_date: Option<Date>,
    pub not_null_array: Vec<i64>,
    pub nullable_array: Option<Vec<i64>>,
    pub nullable_string: Option<String>,
    #[spanner(commitTimestamp)]
    pub updated_at: OffsetDateTime,
}

impl Default for User {
    fn default() -> Self {
        Self {
            not_null_timestamp: OffsetDateTime::UNIX_EPOCH,
            not_null_date: OffsetDateTime::UNIX_EPOCH.date(),
            updated_at: OffsetDateTime::UNIX_EPOCH,
            user_id: Default::default(),
            not_null_int64: Default::default(),
            nullable_int64: Default::default(),
            not_null_float64: Default::default(),
            nullable_float64: Default::default(),
            not_null_bool: Default::default(),
            nullable_bool: Default::default(),
            not_null_byte_array: Default::default(),
            nullable_byte_array: Default::default(),
            not_null_numeric: BigDecimal::zero(),
            nullable_numeric: Default::default(),
            nullable_timestamp: Default::default(),
            nullable_date: Default::default(),
            not_null_array: Default::default(),
            nullable_array: Default::default(),
            nullable_string: Default::default(),
        }
    }
}

#[derive(Query)]
pub struct UserBundle {
    pub user_id: String,
    pub user_characters: Vec<UserCharacter>,
    pub user_items: Vec<UserItem>,
}

#[tokio::test]
#[serial]
async fn test_table_derive() -> Result<(), Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let config = ClientConfig::default();
    let client = Client::new(
        "projects/local-project/instances/test-instance/databases/local-database",
        config,
    )
    .await?;

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let user_id = format!("user{now}");
    let user = User {
        user_id: user_id.clone(),
        not_null_numeric: BigDecimal::from_str("-99999999999999999999999999999.999999999").unwrap(),
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
        assert_eq!(&v.not_null_numeric.to_string(), "-99999999999999999999999999999.999999999");
        assert!(v.updated_at.unix_timestamp() >= now);
        let json_string = serde_json::to_string(&v).unwrap();
        let des = serde_json::from_str::<User>(json_string.as_str()).unwrap();
        assert_eq!(des, v);
    } else {
        panic!("no data found");
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_query_derive() -> Result<(), Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let config = ClientConfig::default();
    let client = Client::new(
        "projects/local-project/instances/test-instance/databases/local-database",
        config,
    )
    .await?;

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let user_id = format!("user-q-{now}");
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

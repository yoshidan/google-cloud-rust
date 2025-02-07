# google-cloud-spanner-derive

Procedural macro for [google-cloud-spanner](../spanner).

[![crates.io](https://img.shields.io/crates/v/gcloud-spanner-derive.svg)](https://crates.io/crates/gcloud-spanner-derive)

## Installation

```toml
[dependencies]
google-cloud-spanner-derive = { package="gcloud-spanner-derive", version="1.0.0" }
```

## Quick Start

### Table derive

`#[derive(Table)]` generates the implementation for following traits.
* `TryFromStruct`
* `ToStruct`
* `TryFrom<Row>`

```rust
use time::OffsetDateTime;
use google_cloud_spanner::client::Client;
use google_cloud_spanner::mutation::insert_struct;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner_derive::Table;

#[derive(Table, Default)]
pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
    // #[spanner(name=...) is used when the column name does not appear in camel case of the field name
    #[spanner(name="LevelX")]
    pub level: i64,
    #[spanner(commitTimestamp)]
    pub updated_at: OffsetDateTime
}

async fn run(client: &Client) -> Result<Vec<UserCharacter>, anyhow::Error> {
    let user = UserCharacter {
        user_id: "user_id".to_string(),
        ..Default::default()
    };
    client.apply(vec![insert_struct("UserCharacter", user)]).await?;

    let mut tx = client.read_only_transaction().await?;
    let stmt = Statement::new("SELECT * From UserCharacter Limit 10");
    let mut reader = tx.query(stmt).await?;
    let mut result = vec![];
    while let Some(row) = reader.next().await? {
        result.push(row.try_into()?);
    }
    Ok(result)
}
```

Here is the generated implementation.
```rust
impl ToStruct for UserCharacter {
    fn to_kinds(&self) -> Kinds {
        vec![
            ("UserId", self.user_id.to_kind()),
            ("CharacterId", self.character_id.to_kind()),
            ("LevelX", self.level.to_kind()),
            ("UpdatedAt", self.updated_at.to_kind()),
        ]
    }

    fn get_types() -> Types {
        vec![
            ("UserId", String::get_type()),
            ("CharacterId", i64::get_type()),
            ("LevelX", i64::get_type()),
            ("UpdatedAt", CommitTimestamp::get_type()),
        ]
    }
}

impl TryFromStruct for UserCharacter {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, RowError> {
        Ok(UserCharacter {
            user_id: s.column_by_name("UserId")?,
            character_id: s.column_by_name("CharacterId")?,
            level: s.column_by_name("LevelX")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}

impl TryFrom<Row> for UserCharacter {
    type Error = RowError;
    fn try_from(s: Row) -> Result<Self, RowError> {
        Ok(UserCharacter {
            user_id: s.column_by_name("UserId")?,
            character_id: s.column_by_name("CharacterId")?,
            level: s.column_by_name("LevelX")?,
            updated_at: s.column_by_name("UpdatedAt")?,
        })
    }
}
```

### Query derive

`#[derive(Query)]` generates the implementation for following traits.
* `TryFrom<Row>`

```rust
use google_cloud_spanner::transaction::Transaction;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner_derive::{Table, Query};

#[derive(Table, Default)]
pub struct UserCharacter {
    pub user_id: String,
    pub character_id: i64,
}

#[derive(Table, Default)]
pub struct UserItem {
    pub user_id: String,
    pub item_id: i64,
}

#[derive(Query, Default)]
pub struct UserBundle {
    pub user_id: String,
    pub user_characters: Vec<UserCharacter>,
    #[spanner(name="Items")]
    pub user_items: Vec<UserItem>
}

async fn run(user_id: &str, tx: &mut Transaction) -> Result<Option<UserBundle>, anyhow::Error> {
    let mut stmt = Statement::new("
        SELECT
            UserId,
            ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @UserId) AS UserCharacters,
            ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = @UserId) AS Items,
        From User WHERE UserID = @UserID",
    );
    stmt.add_param("UserID", &user_id);
    let mut reader = tx.query(stmt).await?;
    match reader.next().await? {
        Some(row) => Ok(row.try_into()?),
        None => Ok(None)
    }
}
```


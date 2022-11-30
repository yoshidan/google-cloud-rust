use chrono::{DateTime, Utc};
use google_cloud_spanner_macro::Table;

#[derive(Table)]
pub struct TableOnly {
    pub id: String
}

#[derive(Table)]
pub struct TableWithColumn {
    pub id: String,
    #[column(name = "OtherName")]
    pub value2: i64,
    pub value3: DateTime::<Utc>,
    pub value4: bool,
    #[column(name = "SpannerUpdatedAt", commitTimestamp)]
    pub updated_at: DateTime::<Utc>,
    #[column(commitTimestamp)]
    pub created_at: DateTime::<Utc>
}

impl TableWithColumn {
    fn new() -> Self {
        Self {
            id: "test".to_string(),
            value2: 20,
            value3: Utc::now(),
            value4: true,
            updated_at: Utc::now(),
            created_at: Utc::now()
        }
    }
}

#[test]
fn test_table_derive() {
    let test = TableOnly{
        id: "test".to_string(),
    };

    let test2 = TableWithColumn::new();
}
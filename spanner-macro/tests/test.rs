use google_cloud_spanner_macro::Table;

#[derive(Table)]
pub struct User {
    pub id: String
}

#[test]
fn test_table_derive() {
    let test = User{
        id: "test".to_string(),
    };
}
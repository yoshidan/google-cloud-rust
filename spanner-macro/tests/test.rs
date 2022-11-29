use google_cloud_spanner_macro::Spanner;

#[derive(Spanner)]
pub struct Table {
    pub id: String
}

#[test]
fn test_spanner_derive() {
    let test = Table{
        id: "test".to_string(),
    };
}
pub mod apiv1;
pub mod client;
pub mod key;
pub mod mutation;
pub mod reader;
pub mod retry;
pub mod row;
pub mod session_pool;
pub mod statement;
pub mod transaction;
pub mod transaction_ro;
pub mod transaction_rw;
pub mod value;

#[cfg(test)]
mod tests_key {
    use crate::key::*;
    use crate::statement::ToKind;
    use google_cloud_googleapis::spanner::*;
    use prost_types::value::Kind;

    #[test]
    fn test_key_new() {
        let mut key = Key::new(vec![true.to_kind()]);
        match key.values.values.pop().unwrap().kind.unwrap() {
            Kind::BoolValue(s) => assert_eq!(s, true),
            _ => panic!("invalid kind"),
        }
    }

    #[test]
    fn test_key_one() {
        let mut key = Key::one(1);
        match key.values.values.pop().unwrap().kind.unwrap() {
            Kind::StringValue(s) => assert_eq!(s, "1"),
            _ => panic!("invalid kind"),
        }
    }

    #[test]
    fn test_key_range() {
        let start = Key::one(1);
        let end = Key::one(100);
        let range = KeyRange::new(start, end, RangeKind::ClosedClosed);
        let raw_range: v1::KeyRange = range.into();
        match raw_range.start_key_type.unwrap() {
            v1::key_range::StartKeyType::StartClosed(mut v) => {
                match v.values.pop().unwrap().kind.unwrap() {
                    Kind::StringValue(v) => assert_eq!(v, "1"),
                    _ => panic!("invalid start kind"),
                }
            }
            _ => panic!("invalid start key trype"),
        }

        match raw_range.end_key_type.unwrap() {
            v1::key_range::EndKeyType::EndClosed(mut v) => {
                match v.values.pop().unwrap().kind.unwrap() {
                    Kind::StringValue(v) => assert_eq!(v, "100"),
                    _ => panic!("invalid end kind"),
                }
            }
            _ => panic!("invalid end key trype"),
        }
    }
}

#[cfg(test)]
mod tests_mutation {
    use crate::key::*;
    use crate::mutation::*;
    use crate::statement::ToKind;
    use crate::value::CommitTimestamp;
    use google_cloud_googleapis::spanner::*;
    use chrono::Utc;
    use crate::key::KeySet;

    #[test]
    fn test_insert() {
        let mutation = insert(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec!["1".to_kind(), "2".to_kind(), CommitTimestamp::from(Utc::now().naive_utc()).to_kind()]);
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Insert(mut w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.len());
                assert_eq!("GuildId", w.columns.pop().unwrap());
                assert_eq!("UserId", w.columns.pop().unwrap());
                assert_eq!("UpdatedAt", w.columns.pop().unwrap());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_update() {
        let mutation = update(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec!["1".to_kind(), "2".to_kind(), CommitTimestamp::from(Utc::now().naive_utc()).to_kind()]);
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Update(w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.len());
                assert_eq!(3, w.columns.len());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_replace() {
        let mutation = replace(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec!["1".to_kind(), "2".to_kind(), CommitTimestamp::from(Utc::now().naive_utc()).to_kind()]);
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Replace(w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.len());
                assert_eq!(3, w.columns.len());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_insert_or_update() {
        let mutation = insert_or_update(
            "Guild",
            vec!["GuildId", "UserId", "UpdatedAt"],
            vec!["1".to_kind(), "2".to_kind(), CommitTimestamp::from(Utc::now().naive_utc()).to_kind()]);
        match mutation.operation.unwrap() {
            v1::mutation::Operation::InsertOrUpdate(w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(3, w.values.len());
                assert_eq!(3, w.columns.len());
            }
            _ => panic!("invalid operation"),
        }
    }

    #[test]
    fn test_delete() {
        let mutation = delete("Guild", all_keys());
        match mutation.operation.unwrap() {
            v1::mutation::Operation::Delete(w) => {
                assert_eq!("Guild", w.table);
                assert_eq!(true, w.key_set.unwrap().all);
            }
            _ => panic!("invalid operation"),
        }
    }
}

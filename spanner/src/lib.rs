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

use prost_types::{ListValue, Value};

use google_cloud_googleapis::spanner::v1::key_range::{EndKeyType, StartKeyType};
use google_cloud_googleapis::spanner::v1::KeyRange as InternalKeyRange;
use google_cloud_googleapis::spanner::v1::KeySet as InternalKeySet;

use crate::statement::ToKind;

/// A Key can be either a Cloud Spanner row's primary key or a secondary index
/// key. A Key can be used as:
///
///   - A primary key which uniquely identifies a Cloud Spanner row.
///   - A secondary index key which maps to a set of Cloud Spanner rows indexed under it.
///   - An endpoint of primary key/secondary index ranges; see the KeyRange type.
///
/// Rows that are identified by the Key type are outputs of read operation or
/// targets of delete operation in a mutation. Note that for
/// insert/update/insert_or_update/delete mutation types, although they don't
/// require a primary key explicitly, the column list provided must contain
/// enough columns that can comprise a primary key.
///
/// Keys are easy to construct.  For example, suppose you have a table with a
/// primary key of username and product ID.  To make a key for this table:
/// ```
/// use google_cloud_spanner::key::Key;
///
/// let key = Key::composite(&[&"john", &16]);
/// ```
/// See the description of Row and Mutation types for how Go types are mapped to
/// Cloud Spanner types. For convenience, Key type supports a range of Rust
/// types:
///   - i64 and Option<i64> are mapped to Cloud Spanner's INT64 type.
///   - f64 and Option<f64> are mapped to Cloud Spanner's FLOAT64 type.
///   - bool and Option<bool> are mapped to Cloud Spanner's BOOL type.
///   - Vec<u8>, &[u8], Option<Vec<u8>> and Option<&[u8]> is mapped to Cloud Spanner's BYTES type.
///   - String, &str, Option<String>, Option<&str> are mapped to Cloud Spanner's STRING type.
///   - time::OffsetDateTime and Option<time::OffsetDateTime> are mapped to Cloud Spanner's TIMESTAMP type.
///   - time::Date and Option<time::Date> are mapped to Cloud Spanner's DATE type.
///   - google_cloud_spanner::value::CommitTimestamp and Option<google_cloud_spanner::value::CommitTimestamp> are mapped to Cloud Spanner's TIMESTAMP type.
#[derive(Clone)]
pub struct Key {
    pub(crate) values: ListValue,
}

/// / A KeySet defines a collection of Cloud Spanner keys and/or key ranges. All
/// / the keys are expected to be in the same table or index. The keys need not be
/// / sorted in any particular way.
/// /
/// / An individual Key can act as a KeySet, as can a KeyRange. Use the KeySets
/// / function to create a KeySet consisting of multiple Keys and KeyRanges. To
/// / obtain an empty KeySet, call KeySets with no arguments.
/// /
/// / If the same key is specified multiple times in the set (for example if two
/// / ranges, two keys, or a key and a range overlap), the Cloud Spanner backend
/// / behaves as if the key were only specified once.
#[derive(Clone)]
pub struct KeySet {
    pub(crate) inner: InternalKeySet,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RangeKind {
    /// ClosedOpen is closed on the left and open on the right: the Start
    /// key is included, the End key is excluded.
    ClosedOpen,

    /// ClosedClosed is closed on the left and the right: both keys are included.
    ClosedClosed,

    /// OpenClosed is open on the left and closed on the right: the Start
    /// key is excluded, the End key is included.
    OpenClosed,

    /// OpenOpen is open on the left and the right: neither key is included.
    OpenOpen,
}

///  A KeyRange represents a range of rows in a table or index.
///
///  A range has a Start key and an End key.  IncludeStart and IncludeEnd
///  indicate whether the Start and End keys are included in the range.
///
///  For example, consider the following table definition:
///
///     CREATE TABLE UserEvents (
///         UserName STRING(MAX),
///         EventDate STRING(10),
///     ) PRIMARY KEY(UserName, EventDate);
///
///  The following keys name rows in this table:
///
///  ```
///    use google_cloud_spanner::key::Key;
///    use google_cloud_spanner::statement::ToKind;
///    let key1 = Key::composite(&[&"Bob", &"2014-09-23"]);
///    let key2 = Key::composite(&[&"Alfred", &"2015-06-12"]);
///  ```
///
///  Since the UserEvents table's PRIMARY KEY clause names two columns, each
///  UserEvents key has two elements; the first is the UserName, and the second
///  is the EventDate.
///
///  Key ranges with multiple components are interpreted lexicographically by
///  component using the table or index key's declared sort order. For example,
///  the following range returns all events for user "Bob" that occurred in the
///  year 2015:
///  ```
///    use google_cloud_spanner::key::{Key, KeyRange, RangeKind};
///    use google_cloud_spanner::statement::ToKind;
///    let range = KeyRange::new(
///        Key::composite(&[&"Bob", &"2015-01-01"]),
///        Key::composite(&[&"Bob", &"2015-12-31"]),
///        RangeKind::ClosedClosed
///    );
///  ```
///
///  Start and end keys can omit trailing key components. This affects the
///  inclusion and exclusion of rows that exactly match the provided key
///  components: if IncludeStart is true, then rows that exactly match the
///  provided components of the Start key are included; if IncludeStart is false
///  then rows that exactly match are not included.  IncludeEnd and End key
///  behave in the same fashion.
///
///  For example, the following range includes all events for "Bob" that occurred
///  during and after the year 2000:
///  ```
///    use google_cloud_spanner::key::{Key, KeyRange, RangeKind};
///    use google_cloud_spanner::statement::ToKind;
///    KeyRange::new(
///     Key::composite(&[&"Bob", &"2000-01-01"]),
///     Key::new(&"Bob"),
///     RangeKind::ClosedClosed
///    );
///  ```
///
///  The next example retrieves all events for "Bob":
///
///     Key::new("Bob").to_prefix()
///
///  To retrieve events before the year 2000:
///  ```
///    use google_cloud_spanner::key::{Key, KeyRange, RangeKind};
///    use google_cloud_spanner::statement::ToKind;
///    let range = KeyRange::new(
///     Key::new(&"Bob"),
///     Key::composite(&[&"Bob", &"2000-01-01"]),
///     RangeKind::ClosedOpen
///    );
///  ```
///
///  Key ranges honor column sort order. For example, suppose a table is defined
///  as follows:
///
///     CREATE TABLE DescendingSortedTable {
///         Key INT64,
///         ...
///     ) PRIMARY KEY(Key DESC);
///
///  The following range retrieves all rows with key values between 1 and 100
///  inclusive:
///
///  ```
///    use google_cloud_spanner::key::{Key, KeyRange, RangeKind};
///    let range = KeyRange::new(
///         Key::new(&100),
///         Key::new(&1),
///    RangeKind::ClosedClosed,
///    );
///  ```
///
///  Note that 100 is passed as the start, and 1 is passed as the end, because
///  Key is a descending column in the schema.
#[derive(Clone)]
pub struct KeyRange {
    /// start specifies the left boundary of the key range;.
    pub(crate) start: Key,

    /// end specifies the right boundary of the key range.
    pub(crate) end: Key,

    /// kind describes whether the boundaries of the key range include
    /// their keys.
    pub kind: RangeKind,
}

/// all_keys returns a KeySet that represents all Keys of a table or a index.
pub fn all_keys() -> KeySet {
    KeySet {
        inner: InternalKeySet {
            keys: vec![],
            ranges: vec![],
            all: true,
        },
    }
}

impl From<KeySet> for InternalKeySet {
    fn from(key_set: KeySet) -> Self {
        key_set.inner
    }
}

impl KeyRange {
    pub fn new(start: Key, end: Key, kind: RangeKind) -> KeyRange {
        KeyRange { start, end, kind }
    }
}

impl From<KeyRange> for InternalKeyRange {
    fn from(key_range: KeyRange) -> Self {
        let (start, end) = match key_range.kind {
            RangeKind::ClosedClosed => (
                Some(StartKeyType::StartClosed(key_range.start.values)),
                Some(EndKeyType::EndClosed(key_range.end.values)),
            ),
            RangeKind::ClosedOpen => (
                Some(StartKeyType::StartClosed(key_range.start.values)),
                Some(EndKeyType::EndOpen(key_range.end.values)),
            ),
            RangeKind::OpenClosed => (
                Some(StartKeyType::StartOpen(key_range.start.values)),
                Some(EndKeyType::EndClosed(key_range.end.values)),
            ),
            RangeKind::OpenOpen => (
                Some(StartKeyType::StartOpen(key_range.start.values)),
                Some(EndKeyType::EndOpen(key_range.end.values)),
            ),
        };
        InternalKeyRange {
            start_key_type: start,
            end_key_type: end,
        }
    }
}

impl From<KeyRange> for KeySet {
    fn from(key_range: KeyRange) -> Self {
        KeySet {
            inner: InternalKeySet {
                keys: vec![],
                ranges: vec![key_range.into()],
                all: false,
            },
        }
    }
}

impl Key {
    /// one creates new Key
    /// # Examples
    /// ```
    ///    use google_cloud_spanner::key::Key;
    ///    use google_cloud_spanner::statement::ToKind;
    ///    let key1 = Key::new(&"a");
    ///    let key2 = Key::new(&1);
    /// ```
    pub fn new(value: &dyn ToKind) -> Key {
        Key::composite(&[value])
    }

    /// one creates new Key
    /// # Examples
    /// ```
    ///    use google_cloud_spanner::key::Key;
    ///    use google_cloud_spanner::statement::ToKind;
    ///    let multi_key = Key::composite(&[&"a", &1]);
    /// ```
    pub fn composite(values: &[&dyn ToKind]) -> Key {
        Key {
            values: ListValue {
                values: values
                    .iter()
                    .map(|x| Value {
                        kind: Some(x.to_kind()),
                    })
                    .collect(),
            },
        }
    }
}

impl From<Key> for KeySet {
    fn from(key: Key) -> Self {
        KeySet {
            inner: InternalKeySet {
                keys: vec![key.values],
                ranges: vec![],
                all: false,
            },
        }
    }
}

impl From<Vec<Key>> for KeySet {
    fn from(keys: Vec<Key>) -> Self {
        let keys = keys.into_iter().map(|key| key.values).collect();
        KeySet {
            inner: InternalKeySet {
                keys,
                ranges: vec![],
                all: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key::*;

    use google_cloud_googleapis::spanner::*;
    use prost_types::value::Kind;

    #[test]
    fn test_key_new() {
        let mut key = Key::new(&true);
        match key.values.values.pop().unwrap().kind.unwrap() {
            Kind::BoolValue(s) => assert!(s),
            _ => panic!("invalid kind"),
        }
    }

    #[test]
    fn test_key_keys() {
        let mut key = Key::composite(&[&true, &1, &"aaa"]);
        match key.values.values.pop().unwrap().kind.unwrap() {
            Kind::StringValue(s) => assert_eq!(s, "aaa"),
            _ => panic!("invalid kind"),
        }
    }

    #[test]
    fn test_key_one() {
        let mut key = Key::new(&1);
        match key.values.values.pop().unwrap().kind.unwrap() {
            Kind::StringValue(s) => assert_eq!(s, "1"),
            _ => panic!("invalid kind"),
        }
    }

    #[test]
    fn test_key_range() {
        let start = Key::new(&1);
        let end = Key::new(&100);
        let range = KeyRange::new(start, end, RangeKind::ClosedClosed);
        let raw_range: v1::KeyRange = range.into();
        match raw_range.start_key_type.unwrap() {
            v1::key_range::StartKeyType::StartClosed(mut v) => match v.values.pop().unwrap().kind.unwrap() {
                Kind::StringValue(v) => assert_eq!(v, "1"),
                _ => panic!("invalid start kind"),
            },
            _ => panic!("invalid start key trype"),
        }

        match raw_range.end_key_type.unwrap() {
            v1::key_range::EndKeyType::EndClosed(mut v) => match v.values.pop().unwrap().kind.unwrap() {
                Kind::StringValue(v) => assert_eq!(v, "100"),
                _ => panic!("invalid end kind"),
            },
            _ => panic!("invalid end key trype"),
        }
    }
}

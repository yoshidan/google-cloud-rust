use internal::spanner::v1::key_range::{EndKeyType, StartKeyType};
use internal::spanner::v1::KeyRange;
use internal::spanner::v1::KeySet as InternalKeySet;
use prost_types::{value, ListValue, Value};

#[derive(Clone)]
pub struct Key {
    pub(crate) values: ListValue,
}

#[derive(Clone)]
pub struct KeySet {
    pub(crate) inner: InternalKeySet,
}

#[derive(Clone)]
pub enum RangeKind {
    ClosedClosed,
    ClosedOpen,
    OpenClosed,
    OpenOpen,
}

#[derive(Clone)]
pub struct Range {
    pub(crate) start: Key,
    pub(crate) end: Key,
    pub kind: RangeKind,
}

pub fn all_keys() -> KeySet {
    KeySet {
        inner: InternalKeySet {
            keys: vec![],
            ranges: vec![],
            all: true,
        }
    }
}

impl Range {
    pub fn new(start: Key, end: Key) -> Range {
        Range {
            start,
            end,
            kind: RangeKind::ClosedClosed,
        }
    }
}

impl From<Range> for KeyRange {
    fn from(key_range: Range) -> Self {
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
        KeyRange {
            start_key_type: start,
            end_key_type: end,
        }
    }
}

impl From<Range> for KeySet {
    fn from(key_range: Range) -> Self {
        KeySet {
            inner: InternalKeySet {
                keys: vec![],
                ranges: vec![key_range.into()],
                all: false,
            }
        }
    }
}

impl Key {
    pub fn new(values: Vec<value::Kind>) -> Key {
        Key {
            values: ListValue {
                values: values
                    .into_iter()
                    .map(|x| Value { kind: Some(x) })
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
            }
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
            }
        }
    }
}

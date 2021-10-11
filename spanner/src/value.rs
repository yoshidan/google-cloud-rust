use chrono::{NaiveDate, NaiveDateTime};
use std::ops::Deref;

pub struct CommitTimestamp {
    pub timestamp: NaiveDateTime,
}

impl Deref for CommitTimestamp {
    type Target = NaiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.timestamp
    }
}

impl From<CommitTimestamp> for NaiveDateTime {
    fn from(s: CommitTimestamp) -> Self {
        s.timestamp
    }
}

impl From<NaiveDateTime> for CommitTimestamp {
    fn from(s: NaiveDateTime) -> Self {
        CommitTimestamp { timestamp: s }
    }
}

use std::ops::Deref;
use std::time::Duration;

use chrono::NaiveDateTime;
use prost_types::Timestamp;

use google_cloud_googleapis::spanner::v1::transaction_options::read_only::TimestampBound as InternalTimestampBound;
use google_cloud_googleapis::spanner::v1::transaction_options::ReadOnly;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
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

#[derive(Clone)]
pub struct TimestampBound {
    inner: InternalTimestampBound,
}

impl TimestampBound {
    pub fn strong_read() -> Self {
        TimestampBound {
            inner: InternalTimestampBound::Strong(true),
        }
    }
    pub fn exact_staleness(d: Duration) -> Self {
        TimestampBound {
            inner: InternalTimestampBound::ExactStaleness(d.into()),
        }
    }
    pub fn max_staleness(d: Duration) -> Self {
        TimestampBound {
            inner: InternalTimestampBound::MaxStaleness(d.into()),
        }
    }
    pub fn min_read_timestamp(t: Timestamp) -> Self {
        TimestampBound {
            inner: InternalTimestampBound::MinReadTimestamp(t.into()),
        }
    }
    pub fn read_timestamp(t: Timestamp) -> Self {
        TimestampBound {
            inner: InternalTimestampBound::ReadTimestamp(t.into()),
        }
    }
}

impl From<TimestampBound> for ReadOnly {
    fn from(tb: TimestampBound) -> Self {
        ReadOnly {
            return_read_timestamp: true,
            timestamp_bound: Some(tb.inner),
        }
    }
}

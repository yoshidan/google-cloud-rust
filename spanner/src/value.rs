use std::ops::Deref;
use std::time::{Duration, SystemTime};

use chrono::{NaiveDateTime, DateTime, Utc, FixedOffset, TimeZone};
use prost_types::Timestamp;

use google_cloud_googleapis::spanner::v1::transaction_options::read_only::TimestampBound as InternalTimestampBound;
use google_cloud_googleapis::spanner::v1::transaction_options::ReadOnly;
use std::fmt::Display;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct CommitTimestamp {
    pub timestamp: DateTime<Utc>,
}

impl CommitTimestamp {
    pub fn new() -> Self {
        CommitTimestamp {
            timestamp: DateTime::<Utc>::from(SystemTime::now())
        }
    }
}

impl Default for CommitTimestamp {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CommitTimestamp {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.timestamp
    }
}

impl From<CommitTimestamp> for DateTime<Utc> {
    fn from(s: CommitTimestamp) -> Self {
        s.timestamp
    }
}

impl From<DateTime<Utc>> for CommitTimestamp {
    fn from(s: DateTime<Utc>) -> Self {
        CommitTimestamp { timestamp: s.into() }
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
            inner: InternalTimestampBound::MinReadTimestamp(t),
        }
    }
    pub fn read_timestamp(t: Timestamp) -> Self {
        TimestampBound {
            inner: InternalTimestampBound::ReadTimestamp(t),
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

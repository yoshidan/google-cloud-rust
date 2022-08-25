use std::ops::Deref;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};

use google_cloud_googleapis::spanner::v1::transaction_options::read_only::TimestampBound as InternalTimestampBound;
use google_cloud_googleapis::spanner::v1::transaction_options::ReadOnly;

#[derive(Clone)]
pub struct SpannerNumeric {
    /// https://cloud.google.com/spanner/docs/storing-numeric-data#precision_of_numeric_types
    /// -99999999999999999999999999999.999999999～99999999999999999999999999999.999999999
    pub inner: String,
}

impl SpannerNumeric {
    pub fn new(value: impl Into<String>) -> Self {
        Self { inner: value.into() }
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    pub nanos: i32,
}

impl From<Timestamp> for prost_types::Timestamp {
    fn from(t: Timestamp) -> Self {
        prost_types::Timestamp {
            seconds: t.seconds,
            nanos: t.nanos,
        }
    }
}

impl From<prost_types::Timestamp> for Timestamp {
    fn from(t: prost_types::Timestamp) -> Self {
        Timestamp {
            seconds: t.seconds,
            nanos: t.nanos,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct CommitTimestamp {
    pub(crate) timestamp: DateTime<Utc>,
}

impl CommitTimestamp {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for CommitTimestamp {
    fn default() -> Self {
        CommitTimestamp {
            timestamp: Utc.timestamp(0, 0),
        }
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

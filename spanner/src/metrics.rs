#[cfg(feature = "otel-metrics")]
use std::collections::HashMap;
#[cfg(feature = "otel-metrics")]
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use google_cloud_gax::grpc::metadata::MetadataMap;
use thiserror::Error;

#[derive(Clone, Default)]
pub struct MetricsConfig {
    /// Enables OpenTelemetry metrics emission when the `otel-metrics` feature is active.
    pub enabled: bool,
    #[cfg(feature = "otel-metrics")]
    pub meter_provider: Option<Arc<dyn opentelemetry::metrics::MeterProvider + Send + Sync>>,
}

impl std::fmt::Debug for MetricsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("MetricsConfig");
        ds.field("enabled", &self.enabled);
        #[cfg(feature = "otel-metrics")]
        {
            let provider = self.meter_provider.is_some();
            ds.field("meter_provider_present", &provider);
        }
        ds.finish()
    }
}

#[derive(Clone, Default)]
pub(crate) struct MetricsRecorder {
    #[cfg(feature = "otel-metrics")]
    inner: Option<Arc<OtelMetrics>>,
}

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("invalid database name: {0}")]
    InvalidDatabase(String),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct SessionPoolSnapshot {
    pub open_sessions: usize,
    pub sessions_in_use: usize,
    pub idle_sessions: usize,
    pub max_allowed_sessions: usize,
    pub max_in_use_last_window: usize,
    pub has_multiplexed_session: bool,
}

pub(crate) type SessionPoolStatsFn = Arc<dyn Fn() -> SessionPoolSnapshot + Send + Sync>;

impl MetricsRecorder {
    pub fn try_new(database: &str, config: &MetricsConfig) -> Result<Self, MetricsError> {
        #[cfg(feature = "otel-metrics")]
        {
            if config.enabled {
                let parsed = parse_database_name(database)?;
                let inner = OtelMetrics::new(parsed, config.meter_provider.clone());
                Ok(Self {
                    inner: Some(Arc::new(inner)),
                })
            } else {
                Ok(Self { inner: None })
            }
        }
        #[cfg(not(feature = "otel-metrics"))]
        {
            let _ = database;
            let _ = config;
            Ok(Self::default())
        }
    }

    pub(crate) fn register_session_pool(&self, stats: SessionPoolStatsFn) {
        #[cfg(feature = "otel-metrics")]
        if let Some(inner) = &self.inner {
            inner.register_session_pool(stats);
        }
        #[cfg(not(feature = "otel-metrics"))]
        {
            let _ = stats;
        }
    }

    pub(crate) fn record_session_timeout(&self) {
        #[cfg(feature = "otel-metrics")]
        if let Some(inner) = &self.inner {
            inner.record_session_timeout();
        }
    }

    pub(crate) fn record_session_acquired(&self) {
        #[cfg(feature = "otel-metrics")]
        if let Some(inner) = &self.inner {
            inner.record_session_acquired();
        }
    }

    pub(crate) fn record_session_released(&self) {
        #[cfg(feature = "otel-metrics")]
        if let Some(inner) = &self.inner {
            inner.record_session_released();
        }
    }

    pub(crate) fn record_session_acquire_latency(&self, duration: Duration) {
        #[cfg(feature = "otel-metrics")]
        if let Some(inner) = &self.inner {
            inner.record_session_acquire_latency(duration);
        }
        #[cfg(not(feature = "otel-metrics"))]
        {
            let _ = duration;
        }
    }

    pub(crate) fn record_server_timing(&self, method: &'static str, metadata: &MetadataMap) {
        #[cfg(feature = "otel-metrics")]
        if let Some(inner) = &self.inner {
            let metrics = parse_server_timing(metadata);
            inner.record_gfe_metrics(method, metrics);
        }
        #[cfg(not(feature = "otel-metrics"))]
        {
            let _ = method;
            let _ = metadata;
        }
    }
}

#[cfg(feature = "otel-metrics")]
mod otel_impl {
    use super::{
        ParsedDatabaseName, ServerTimingMetrics, SessionPoolStatsFn, ATTR_CLIENT_ID, ATTR_DATABASE, ATTR_INSTANCE,
        ATTR_IS_MULTIPLEXED, ATTR_LIB_VERSION, ATTR_METHOD, ATTR_PROJECT, ATTR_TYPE, CLIENT_ID_SEQ, GFE_BUCKETS,
        GFE_TIMING_HEADER, METRICS_PREFIX, OTEL_SCOPE, SESSION_ACQUIRE_BUCKETS,
    };
    use opentelemetry::metrics::{Counter, Histogram, Meter, MeterProvider, ObservableGauge};
    use opentelemetry::{global, InstrumentationScope, KeyValue};
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    pub(super) struct OtelMetrics {
        meter: Meter,
        attributes: AttributeSets,
        session_gauges: Mutex<Option<SessionGaugeHandles>>,
        get_session_timeouts: Counter<u64>,
        acquired_sessions: Counter<u64>,
        released_sessions: Counter<u64>,
        gfe_latency: Histogram<f64>,
        gfe_header_missing: Counter<u64>,
        session_acquire_latency: Histogram<f64>,
    }

    struct SessionGaugeHandles {
        _open_session_count: ObservableGauge<i64>,
        _max_allowed_sessions: ObservableGauge<i64>,
        _num_sessions: ObservableGauge<i64>,
        _max_in_use_sessions: ObservableGauge<i64>,
    }

    impl OtelMetrics {
        pub(super) fn new(
            parsed: ParsedDatabaseName,
            meter_provider: Option<Arc<dyn MeterProvider + Send + Sync>>,
        ) -> Self {
            let scope = InstrumentationScope::builder(OTEL_SCOPE)
                .with_version(env!("CARGO_PKG_VERSION"))
                .build();
            let meter = if let Some(provider) = meter_provider {
                provider.meter_with_scope(scope)
            } else {
                global::meter_provider().meter_with_scope(scope)
            };

            let attributes = AttributeSets::new(parsed);

            let get_session_timeouts = meter
                .u64_counter(METRICS_PREFIX.to_owned() + "get_session_timeouts")
                .with_description("The number of get sessions timeouts due to pool exhaustion.")
                .with_unit("1")
                .build();
            let acquired_sessions = meter
                .u64_counter(METRICS_PREFIX.to_owned() + "num_acquired_sessions")
                .with_description("The number of sessions acquired from the session pool.")
                .with_unit("1")
                .build();
            let released_sessions = meter
                .u64_counter(METRICS_PREFIX.to_owned() + "num_released_sessions")
                .with_description("The number of sessions released by the user and pool maintainer.")
                .with_unit("1")
                .build();
            let gfe_latency = meter
                .f64_histogram(METRICS_PREFIX.to_owned() + "gfe_latency")
                .with_description("Latency between Google's network receiving an RPC and reading back the first byte of the response.")
                .with_unit("ms")
                .with_boundaries(GFE_BUCKETS.to_vec())
                .build();
            let gfe_header_missing = meter
                .u64_counter(METRICS_PREFIX.to_owned() + "gfe_header_missing_count")
                .with_description("Number of RPC responses received without the server-timing header, most likely meaning the RPC never reached Google's network.")
                .with_unit("1")
                .build();
            let session_acquire_latency = meter
                .f64_histogram(METRICS_PREFIX.to_owned() + "session_acquire_latency")
                .with_description("Time spent waiting to acquire a session from the pool.")
                .with_unit("ms")
                .with_boundaries(SESSION_ACQUIRE_BUCKETS.to_vec())
                .build();

            OtelMetrics {
                meter,
                attributes,
                session_gauges: Mutex::new(None),
                get_session_timeouts,
                acquired_sessions,
                released_sessions,
                gfe_latency,
                gfe_header_missing,
                session_acquire_latency,
            }
        }

        pub(super) fn register_session_pool(&self, stats: SessionPoolStatsFn) {
            let mut guard = self.session_gauges.lock().unwrap();
            if guard.is_some() {
                return;
            }

            let open_stats = stats.clone();
            let base = self.attributes.base.clone();
            let multiplexed = self.attributes.with_multiplexed.clone();
            let open_session_count = self
                .meter
                .i64_observable_gauge(METRICS_PREFIX.to_owned() + "open_session_count")
                .with_description("Number of sessions currently opened.")
                .with_unit("1")
                .with_callback(move |observer| {
                    let snapshot = open_stats();
                    if snapshot.has_multiplexed_session {
                        observer.observe(1, multiplexed.as_ref());
                    }
                    observer.observe(snapshot.open_sessions as i64, base.as_ref());
                })
                .build();

            let max_allowed_stats = stats.clone();
            let base = self.attributes.base.clone();
            let max_allowed_sessions = self
                .meter
                .i64_observable_gauge(METRICS_PREFIX.to_owned() + "max_allowed_sessions")
                .with_description("The maximum number of sessions allowed. Configurable by the user.")
                .with_unit("1")
                .with_callback(move |observer| {
                    let snapshot = max_allowed_stats();
                    observer.observe(snapshot.max_allowed_sessions as i64, base.as_ref());
                })
                .build();

            let sessions_stats = stats.clone();
            let in_use_attrs = self.attributes.num_in_use.clone();
            let idle_attrs = self.attributes.num_sessions.clone();
            let num_sessions = self
                .meter
                .i64_observable_gauge(METRICS_PREFIX.to_owned() + "num_sessions_in_pool")
                .with_description("The number of sessions currently in use.")
                .with_unit("1")
                .with_callback(move |observer| {
                    let snapshot = sessions_stats();
                    observer.observe(snapshot.sessions_in_use as i64, in_use_attrs.as_ref());
                    observer.observe(snapshot.idle_sessions as i64, idle_attrs.as_ref());
                })
                .build();

            let max_in_use_stats = stats;
            let attrs = self.attributes.max_in_use.clone();
            let max_in_use_sessions = self
                .meter
                .i64_observable_gauge(METRICS_PREFIX.to_owned() + "max_in_use_sessions")
                .with_description("The maximum number of sessions in use during the last 10 minute interval.")
                .with_unit("1")
                .with_callback(move |observer| {
                    let snapshot = max_in_use_stats();
                    observer.observe(snapshot.max_in_use_last_window as i64, attrs.as_ref());
                })
                .build();

            *guard = Some(SessionGaugeHandles {
                _open_session_count: open_session_count,
                _max_allowed_sessions: max_allowed_sessions,
                _num_sessions: num_sessions,
                _max_in_use_sessions: max_in_use_sessions,
            });
        }

        pub(super) fn record_session_timeout(&self) {
            self.get_session_timeouts
                .add(1, self.attributes.without_multiplexed.as_ref());
        }

        pub(super) fn record_session_acquired(&self) {
            self.acquired_sessions
                .add(1, self.attributes.without_multiplexed.as_ref());
        }

        pub(super) fn record_session_released(&self) {
            self.released_sessions
                .add(1, self.attributes.without_multiplexed.as_ref());
        }

        pub(super) fn record_session_acquire_latency(&self, duration: Duration) {
            let latency_ms = duration.as_secs_f64() * 1000.0;
            self.session_acquire_latency
                .record(latency_ms, self.attributes.base.as_ref());
        }

        pub(super) fn record_gfe_metrics(&self, method: &'static str, metrics: ServerTimingMetrics) {
            if metrics.is_empty() {
                self.gfe_header_missing.add(1, self.attributes.base.as_ref());
                return;
            }

            let mut attrs: Vec<KeyValue> = self.attributes.base.as_ref().to_vec();
            attrs.push(KeyValue::new(ATTR_METHOD, method));

            let latency = metrics.value(GFE_TIMING_HEADER);
            self.gfe_latency.record(latency, &attrs);
        }
    }

    #[derive(Clone)]
    struct AttributeSets {
        base: Arc<[KeyValue]>,
        with_multiplexed: Arc<[KeyValue]>,
        without_multiplexed: Arc<[KeyValue]>,
        num_in_use: Arc<[KeyValue]>,
        num_sessions: Arc<[KeyValue]>,
        max_in_use: Arc<[KeyValue]>,
    }

    impl AttributeSets {
        fn new(parsed: ParsedDatabaseName) -> Self {
            let client_id = next_client_id();
            let base_vec = vec![
                KeyValue::new(ATTR_CLIENT_ID, client_id),
                KeyValue::new(ATTR_DATABASE, parsed.database),
                KeyValue::new(ATTR_INSTANCE, parsed.instance),
                KeyValue::new(ATTR_PROJECT, parsed.project),
                KeyValue::new(ATTR_LIB_VERSION, env!("CARGO_PKG_VERSION")),
            ];

            let mut with_multiplexed_vec = base_vec.clone();
            with_multiplexed_vec.push(KeyValue::new(ATTR_IS_MULTIPLEXED, "true"));

            let mut without_multiplexed_vec = base_vec.clone();
            without_multiplexed_vec.push(KeyValue::new(ATTR_IS_MULTIPLEXED, "false"));

            let mut num_in_use_vec = without_multiplexed_vec.clone();
            num_in_use_vec.push(KeyValue::new(ATTR_TYPE, "num_in_use_sessions"));

            let mut num_sessions_vec = without_multiplexed_vec.clone();
            num_sessions_vec.push(KeyValue::new(ATTR_TYPE, "num_sessions"));

            let max_in_use_vec = without_multiplexed_vec.clone();

            AttributeSets {
                base: base_vec.into(),
                with_multiplexed: with_multiplexed_vec.into(),
                without_multiplexed: without_multiplexed_vec.into(),
                num_in_use: num_in_use_vec.into(),
                num_sessions: num_sessions_vec.into(),
                max_in_use: max_in_use_vec.into(),
            }
        }
    }

    fn next_client_id() -> String {
        let id = CLIENT_ID_SEQ.fetch_add(1, Ordering::Relaxed);
        format!("rust-client-{id}")
    }
}

#[cfg(feature = "otel-metrics")]
use otel_impl::*;

#[cfg(feature = "otel-metrics")]
const OTEL_SCOPE: &str = "cloud.google.com/go";
#[cfg(feature = "otel-metrics")]
const METRICS_PREFIX: &str = "spanner/";
#[cfg(feature = "otel-metrics")]
const ATTR_CLIENT_ID: &str = "client_id";
#[cfg(feature = "otel-metrics")]
const ATTR_DATABASE: &str = "database";
#[cfg(feature = "otel-metrics")]
const ATTR_INSTANCE: &str = "instance_id";
#[cfg(feature = "otel-metrics")]
const ATTR_PROJECT: &str = "project_id";
#[cfg(feature = "otel-metrics")]
const ATTR_LIB_VERSION: &str = "library_version";
#[cfg(feature = "otel-metrics")]
const ATTR_IS_MULTIPLEXED: &str = "is_multiplexed";
#[cfg(feature = "otel-metrics")]
const ATTR_TYPE: &str = "type";
#[cfg(feature = "otel-metrics")]
const ATTR_METHOD: &str = "grpc_client_method";
#[cfg(feature = "otel-metrics")]
const GFE_TIMING_HEADER: &str = "gfet4t7";
#[cfg(feature = "otel-metrics")]
const SERVER_TIMING_HEADER: &str = "server-timing";

#[cfg(feature = "otel-metrics")]
const SESSION_ACQUIRE_BUCKETS: &[f64] = &[
    0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 75.0, 100.0, 150.0, 200.0, 300.0, 400.0, 500.0, 750.0, 1000.0, 1500.0,
    2000.0, 3000.0, 4000.0, 5000.0, 7500.0, 10000.0, 15000.0, 30000.0, 60000.0,
];

#[cfg(feature = "otel-metrics")]
const GFE_BUCKETS: &[f64] = &[
    0.0, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
    20.0, 25.0, 30.0, 40.0, 50.0, 65.0, 80.0, 100.0, 130.0, 160.0, 200.0, 250.0, 300.0, 400.0, 500.0, 650.0, 800.0,
    1000.0, 2000.0, 5000.0, 10000.0, 20000.0, 50000.0, 100000.0, 200000.0, 400000.0, 800000.0, 1600000.0, 3200000.0,
];

#[cfg(feature = "otel-metrics")]
static CLIENT_ID_SEQ: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "otel-metrics")]
#[derive(Clone)]
struct ParsedDatabaseName {
    database: String,
    instance: String,
    project: String,
}

#[cfg(feature = "otel-metrics")]
fn parse_database_name(name: &str) -> Result<ParsedDatabaseName, MetricsError> {
    let parts: Vec<&str> = name.split('/').collect();
    if parts.len() != 6 || parts[0] != "projects" || parts[2] != "instances" || parts[4] != "databases" {
        return Err(MetricsError::InvalidDatabase(name.to_string()));
    }
    Ok(ParsedDatabaseName {
        project: parts[1].to_string(),
        instance: parts[3].to_string(),
        database: parts[5].to_string(),
    })
}

#[cfg(feature = "otel-metrics")]
#[derive(Clone)]
struct ServerTimingMetrics {
    values: HashMap<String, f64>,
}

#[cfg(feature = "otel-metrics")]
impl ServerTimingMetrics {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn value(&self, key: &str) -> f64 {
        self.values.get(key).copied().unwrap_or_default()
    }
}

#[cfg(feature = "otel-metrics")]
fn parse_server_timing(metadata: &MetadataMap) -> ServerTimingMetrics {
    let mut map = HashMap::new();
    for value in metadata.get_all(SERVER_TIMING_HEADER).iter() {
        if let Ok(raw) = value.to_str() {
            for part in raw.split(',') {
                let trimmed = part.trim();
                if let Some((name, dur_part)) = trimmed.split_once(';') {
                    let name = name.trim();
                    if let Some(duration) = dur_part.trim().strip_prefix("dur=") {
                        if let Ok(parsed) = duration.trim().parse::<f64>() {
                            map.insert(name.to_string(), parsed);
                        }
                    }
                }
            }
        }
    }
    ServerTimingMetrics { values: map }
}

#[cfg(all(test, feature = "otel-metrics"))]
mod tests {
    use super::*;

    #[test]
    fn parses_server_timing_header() {
        let mut metadata = MetadataMap::new();
        metadata
            .insert(SERVER_TIMING_HEADER, "gfet4t7;dur=12.5,another-metric;dur=3.5".parse().unwrap())
            .unwrap();
        let metrics = parse_server_timing(&metadata);
        assert!(!metrics.is_empty());
        assert!((metrics.value(GFE_TIMING_HEADER) - 12.5).abs() < f64::EPSILON);
    }
}

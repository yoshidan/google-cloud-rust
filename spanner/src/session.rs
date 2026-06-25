use std::collections::VecDeque;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::{Mutex, RwLock};
use thiserror;
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{mpsc, oneshot};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;

use tracing::{Instrument, Span};

use google_cloud_gax::grpc::metadata::MetadataMap;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::TryAs;
use google_cloud_googleapis::spanner::v1::{BatchCreateSessionsRequest, CreateSessionRequest, DeleteSessionRequest, Session};

use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::spanner_client::{ping_query_request, Client};
use crate::metrics::{MetricsRecorder, SessionPoolSnapshot, SessionPoolStatsFn};

const MAX_IN_USE_WINDOW: Duration = Duration::from_secs(600);

/// Carried by clones of the multiplexed master so that `invalidate_if_needed`
/// can flip the manager's invalid flag if and only if the failing clone
/// belongs to the master generation that is currently installed. Without the
/// generation check, a delayed `NotFound` from a stale clone (whose master
/// has already been recreated) would falsely re-flag the new master as
/// invalid and trigger another redundant `CreateSession` RPC.
pub(crate) struct MultiplexedInvalidator {
    flag: Arc<AtomicBool>,
    clone_generation: u64,
    current_generation: Arc<AtomicU64>,
}

/// Session
pub struct SessionHandle {
    pub session: Session,
    pub spanner_client: Client,
    valid: bool,
    deleted: bool,
    last_used_at: Instant,
    last_checked_at: Instant,
    last_pong_at: Instant,
    created_at: Instant,
    /// Set on clones of a multiplexed session. When `invalidate_if_needed`
    /// observes a NotFound on a multiplexed handle, this flag is raised so
    /// `SessionManager::get` recreates the underlying session on the next
    /// call (subject to a generation check, see `MultiplexedInvalidator`).
    /// Multiplexed sessions cannot be deleted (per the v1 proto), so the
    /// delete RPC is skipped in that case.
    multiplexed_invalidator: Option<MultiplexedInvalidator>,
}

impl SessionHandle {
    pub(crate) fn new(session: Session, spanner_client: Client, now: Instant) -> SessionHandle {
        SessionHandle {
            session,
            spanner_client,
            valid: true,
            deleted: false,
            last_used_at: now,
            last_checked_at: now,
            last_pong_at: now,
            created_at: now,
            multiplexed_invalidator: None,
        }
    }

    pub(crate) fn new_multiplexed_clone(
        session: Session,
        spanner_client: Client,
        now: Instant,
        invalidator: MultiplexedInvalidator,
    ) -> SessionHandle {
        SessionHandle {
            session,
            spanner_client,
            valid: true,
            deleted: false,
            last_used_at: now,
            last_checked_at: now,
            last_pong_at: now,
            created_at: now,
            multiplexed_invalidator: Some(invalidator),
        }
    }

    pub async fn invalidate_if_needed<T>(&mut self, arg: Result<T, Status>) -> Result<T, Status> {
        match arg {
            Ok(s) => Ok(s),
            Err(e) => {
                if e.code() == Code::NotFound && e.message().contains("Session not found:") {
                    if let Some(inv) = &self.multiplexed_invalidator {
                        // Multiplexed sessions cannot be deleted via the API;
                        // signal SessionManager to recreate on the next get(),
                        // but only if this clone still belongs to the master
                        // generation currently installed. A `NotFound` from a
                        // stale clone (master already rotated) is silently
                        // swallowed so it does not trigger a redundant
                        // CreateSession.
                        let current = inv.current_generation.load(Ordering::Acquire);
                        if current == inv.clone_generation {
                            tracing::debug!("multiplexed session invalidated: {}", self.session.name);
                            self.valid = false;
                            inv.flag.store(true, Ordering::Release);
                        } else {
                            tracing::debug!(
                                "stale multiplexed clone NotFound ignored (clone_gen={}, current_gen={}, name={})",
                                inv.clone_generation,
                                current,
                                self.session.name,
                            );
                            self.valid = false;
                        }
                    } else {
                        tracing::debug!("session invalidate {}", self.session.name);
                        self.delete().await;
                    }
                }
                Err(e)
            }
        }
    }

    async fn delete(&mut self) {
        self.valid = false;
        let session_name = &self.session.name;
        let request = DeleteSessionRequest {
            name: session_name.to_string(),
        };
        match self.spanner_client.delete_session(request, true, None).await {
            Ok(_) => self.deleted = true,
            Err(e) => tracing::warn!("failed to delete session {}, {:?}", session_name, e),
        };
    }
}

/// ManagedSession
pub struct ManagedSession {
    session_pool: SessionPool,
    session: Option<SessionHandle>,
    /// When false (multiplexed session mode), the session is NOT returned to
    /// the pool on drop — it is a short-lived clone of a shared handle.
    recycle_on_drop: bool,
}

impl ManagedSession {
    fn new(session_pool: SessionPool, session: SessionHandle) -> Self {
        ManagedSession {
            session_pool,
            session: Some(session),
            recycle_on_drop: true,
        }
    }

    fn new_multiplexed(session_pool: SessionPool, session: SessionHandle) -> Self {
        ManagedSession {
            session_pool,
            session: Some(session),
            recycle_on_drop: false,
        }
    }
}

impl Drop for ManagedSession {
    fn drop(&mut self) {
        let session = self.session.take().unwrap();
        if self.recycle_on_drop {
            self.session_pool.recycle(session);
        } else {
            // Multiplexed sessions are shared; the handle is simply dropped.
            // recycle() emits record_session_released for pool sessions, so
            // emit it directly here to keep acquire/release counts balanced
            // for observability.
            self.session_pool.metrics.record_session_released();
        }
    }
}

impl Deref for ManagedSession {
    type Target = SessionHandle;

    fn deref(&self) -> &Self::Target {
        self.session.as_ref().unwrap()
    }
}

impl DerefMut for ManagedSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.session.as_mut().unwrap()
    }
}

/// Sessions have all sessions and waiters.
/// This is for atomically locking the waiting list and free sessions.
struct Sessions {
    available_sessions: VecDeque<SessionHandle>,

    waiters: VecDeque<oneshot::Sender<()>>,

    /// Invalid sessions living in the server.
    orphans: Vec<SessionHandle>,

    /// number of sessions user uses.
    num_inuse: usize,

    /// number of sessions scheduled to be replenished.
    num_creating: usize,

    /// Maximum observed number of sessions in use during the current window.
    max_inuse_window: usize,
    /// Start of the rolling window used for `max_inuse_window`.
    window_started_at: Instant,
}

impl Sessions {
    fn num_opened(&self) -> usize {
        self.num_inuse + self.available_sessions.len()
    }

    fn take_waiter(&mut self) -> Option<oneshot::Sender<()>> {
        while let Some(waiter) = self.waiters.pop_front() {
            // Waiter can be closed when session acquisition times out.
            if !waiter.is_closed() {
                return Some(waiter);
            }
        }
        None
    }

    fn take(&mut self) -> Option<SessionHandle> {
        match self.available_sessions.pop_front() {
            None => None,
            Some(s) => {
                self.num_inuse += 1;
                self.update_max_in_use();
                Some(s)
            }
        }
    }

    fn release(&mut self, session: SessionHandle) {
        if self.num_inuse > 0 {
            self.num_inuse -= 1;
        }
        if session.valid {
            self.available_sessions.push_back(session);
        } else if !session.deleted {
            tracing::trace!("save as orphan name={}", session.session.name);
            self.orphans.push(session);
        }
    }

    /// reserve calculates next session count to create.
    /// Must call replenish after calling this method.
    fn reserve(&mut self, max_opened: usize, inc_step: usize) -> usize {
        let num_opened = self.num_opened();
        let num_creating = self.num_creating;
        if max_opened < num_creating + num_opened {
            tracing::trace!(
                "No available connections max={}, num_creating={}, current={}",
                max_opened,
                num_creating,
                num_opened
            );
            return 0;
        }
        let mut increasing = max_opened - (num_creating + num_opened);
        if increasing > inc_step {
            increasing = inc_step
        }
        self.num_creating += increasing;
        increasing
    }

    fn replenish(&mut self, session_count: usize, result: Result<Vec<SessionHandle>, Status>) {
        self.num_creating -= session_count;
        match result {
            Ok(mut new_sessions) => {
                while let Some(session) = new_sessions.pop() {
                    self.available_sessions.push_back(session);
                    if let Some(waiter) = self.take_waiter() {
                        let _ = waiter.send(());
                    }
                }
            }
            Err(e) => tracing::error!("failed to create new sessions {:?}", e),
        }
    }

    fn update_max_in_use(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.window_started_at) >= MAX_IN_USE_WINDOW {
            self.window_started_at = now;
            self.max_inuse_window = self.num_inuse;
        } else if self.num_inuse > self.max_inuse_window {
            self.max_inuse_window = self.num_inuse;
        }
    }
}

#[derive(Clone)]
struct SessionPool {
    inner: Arc<RwLock<Sessions>>,
    session_creation_sender: UnboundedSender<usize>,
    config: Arc<SessionConfig>,
    metrics: Arc<MetricsRecorder>,
}

impl SessionPool {
    async fn new(
        database: String,
        conn_pool: &ConnectionManager,
        session_creation_sender: UnboundedSender<usize>,
        config: Arc<SessionConfig>,
        disable_route_to_leader: bool,
        metrics: Arc<MetricsRecorder>,
    ) -> Result<Self, Status> {
        let available_sessions =
            Self::init_pool(database, conn_pool, config.min_opened, disable_route_to_leader, metrics.clone()).await?;
        let pool = SessionPool {
            inner: Arc::new(RwLock::new(Sessions {
                available_sessions,
                waiters: VecDeque::new(),
                orphans: Vec::new(),
                num_inuse: 0,
                num_creating: 0,
                max_inuse_window: 0,
                window_started_at: Instant::now(),
            })),
            session_creation_sender,
            config,
            metrics,
        };
        Ok(pool)
    }

    async fn init_pool(
        database: String,
        conn_pool: &ConnectionManager,
        min_opened: usize,
        disable_route_to_leader: bool,
        metrics: Arc<MetricsRecorder>,
    ) -> Result<VecDeque<SessionHandle>, Status> {
        let channel_num = conn_pool.num();
        let creation_count_per_channel = min_opened / channel_num;
        let remainder = min_opened % channel_num;

        let mut sessions = Vec::<SessionHandle>::new();
        let mut tasks = JoinSet::new();
        for i in 0..channel_num {
            // Ensure that we create the exact number of requested sessions by adding the remainder to the first channel.
            let creation_count = if i == 0 {
                creation_count_per_channel + remainder
            } else {
                creation_count_per_channel
            };
            let next_client = conn_pool
                .conn()
                .with_metrics(metrics.clone())
                .with_metadata(client_metadata(&database));
            let database = database.clone();
            tasks.spawn(async move {
                batch_create_sessions(next_client, &database, creation_count, disable_route_to_leader).await
            }.instrument(Span::current()));
        }
        while let Some(r) = tasks.join_next().await {
            let new_sessions = r.map_err(|e| Status::from_error(e.into()))??;
            sessions.extend(new_sessions);
        }
        tracing::debug!("initial session created count = {}", sessions.len());
        Ok(sessions.into())
    }

    fn num_opened(&self) -> usize {
        self.inner.read().num_opened()
    }

    /// The client first checks the waiting list.
    /// If the waiting list is empty, it retrieves the first available session.
    /// If there are no available sessions, it enters the waiting list.
    /// If the waiting list is not empty, the client enters the waiting list.
    /// The client on the waiting list will be notified when another client's session has finished and
    /// when the process of replenishing the available sessions is complete.
    async fn acquire(&self) -> Result<ManagedSession, SessionError> {
        let request_started_at = Instant::now();
        loop {
            let (on_session_acquired, session_count) = {
                let mut sessions = self.inner.write();

                // Prioritize waiters over new acquirers.
                if sessions.waiters.is_empty() {
                    if let Some(mut s) = sessions.take() {
                        s.last_used_at = Instant::now();
                        self.metrics.record_session_acquired();
                        self.metrics
                            .record_session_acquire_latency(request_started_at.elapsed());
                        return Ok(ManagedSession::new(self.clone(), s));
                    }
                }
                // Add the participant to the waiting list.
                let (sender, receiver) = oneshot::channel();
                sessions.waiters.push_back(sender);
                let session_count = sessions.reserve(self.config.max_opened, self.config.inc_step);
                (receiver, session_count)
            };

            if session_count > 0 {
                let _ = self.session_creation_sender.send(session_count);
            }

            // Wait for the session available notification.
            match timeout(self.config.session_get_timeout, on_session_acquired).await {
                Ok(Ok(())) => {
                    let mut sessions = self.inner.write();
                    if let Some(mut s) = sessions.take() {
                        s.last_used_at = Instant::now();
                        self.metrics.record_session_acquired();
                        self.metrics
                            .record_session_acquire_latency(request_started_at.elapsed());
                        return Ok(ManagedSession::new(self.clone(), s));
                    } else {
                        continue; // another waiter raced for session
                    }
                }
                _ => {
                    {
                        let sessions = self.inner.write();
                        tracing::info!(
                            available = sessions.available_sessions.len(),
                            waiters = sessions.waiters.len(),
                            orphans = sessions.orphans.len(),
                            num_inuse = sessions.num_inuse,
                            num_creating = sessions.num_creating,
                            max_opened = self.config.max_opened,
                            "Timeout acquiring session"
                        );
                    }
                    self.metrics.record_session_timeout();
                    return Err(SessionError::SessionGetTimeout);
                }
            }
        }
    }

    /// If the session is valid
    ///  - Pass the session to the first user on the waiting list.
    ///  - If there is no waiting list, the session is returned to the list of available sessions.
    ///    If the session is invalid
    ///  - Discard the session. If the number of sessions falls below the threshold as a result of discarding, the session replenishment process is called.
    fn recycle(&self, mut session: SessionHandle) {
        self.metrics.record_session_released();
        if session.valid {
            let mut sessions = self.inner.write();
            let waiter = sessions.take_waiter();
            if sessions.num_opened() > self.config.max_idle
                && session.created_at + self.config.idle_timeout < Instant::now()
                && waiter.is_none()
            {
                // Not reuse expired idle session
                session.valid = false
            }
            sessions.release(session);
            if let Some(waiter) = waiter {
                let _ = waiter.send(());
            }
        } else {
            let session_count = {
                let mut sessions = self.inner.write();
                sessions.release(session);
                if sessions.num_opened() < self.config.min_opened && !sessions.waiters.is_empty() {
                    sessions.reserve(self.config.max_opened, self.config.inc_step)
                } else {
                    0
                }
            };
            if session_count > 0 {
                let _ = self.session_creation_sender.send(session_count);
            }
        }
    }

    async fn close(&self) {
        let empty = VecDeque::new();
        let deleting_sessions = { mem::replace(&mut self.inner.write().available_sessions, empty) };
        for mut session in deleting_sessions {
            session.delete().await;
        }

        self.remove_orphans().await;
    }

    fn snapshot_fn(&self, has_multiplexed_session: bool) -> SessionPoolStatsFn {
        let inner = self.inner.clone();
        let max_allowed = self.config.max_opened;
        Arc::new(move || {
            let sessions = inner.read();
            SessionPoolSnapshot {
                open_sessions: sessions.num_opened(),
                sessions_in_use: sessions.num_inuse,
                idle_sessions: sessions.available_sessions.len(),
                max_allowed_sessions: max_allowed,
                max_in_use_last_window: sessions.max_inuse_window,
                has_multiplexed_session,
            }
        })
    }

    async fn remove_orphans(&self) {
        let empty = vec![];
        let deleting_sessions = { mem::replace(&mut self.inner.write().orphans, empty) };
        tracing::trace!("remove {} orphan sessions", deleting_sessions.len());
        for mut session in deleting_sessions {
            session.delete().await;
        }
    }
}

#[derive(Clone, Debug)]
pub struct SessionConfig {
    /// max_opened is the maximum number of opened sessions allowed by the session
    /// pool. If the client tries to open a session and there are already
    /// max_opened sessions, it will block until one becomes available or the
    /// context passed to the client method is canceled or times out.
    pub max_opened: usize,

    /// min_opened is the minimum number of opened sessions that the session pool
    /// tries to maintain. Session pool won't continue to expire sessions if
    /// number of opened connections drops below min_opened. However, if a session
    /// is found to be broken, it will still be evicted from the session pool,
    /// therefore it is posssible that the number of opened sessions drops below
    /// min_opened.
    pub min_opened: usize,

    /// max_idle is the maximum number of idle sessions, pool is allowed to keep.
    pub max_idle: usize,

    /// idle_timeout is the wait time before discarding an idle session.
    /// Sessions older than this value since they were last used will be discarded.
    /// However, if the number of sessions is less than or equal to min_opened, it will not be discarded.
    pub idle_timeout: Duration,

    pub session_alive_trust_duration: Duration,

    /// session_get_timeout is the maximum value of the waiting time that occurs when retrieving from the connection pool when there is no idle session.
    pub session_get_timeout: Duration,

    /// refresh_interval is the interval of cleanup and health check functions.
    pub refresh_interval: Duration,

    /// incStep is the number of sessions to create in one batch when at least
    /// one more session is needed.
    inc_step: usize,

    /// use_multiplexed_session enables multiplexed session mode.
    ///
    /// When true, a single multiplexed session is created via `CreateSession`
    /// (with `Session.multiplexed = true`) and shared across all callers.
    /// No session pool is maintained; the session does not expire.
    ///
    /// Required for Spanner Omni deployments, which only accept multiplexed
    /// sessions. Set automatically when `SPANNER_OMNI_ENDPOINT` is detected.
    pub use_multiplexed_session: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        SessionConfig {
            max_opened: 400,
            min_opened: 10,
            max_idle: 300,
            inc_step: 25,
            idle_timeout: Duration::from_secs(30 * 60),
            session_alive_trust_duration: Duration::from_secs(55 * 60),
            session_get_timeout: Duration::from_secs(1),
            refresh_interval: Duration::from_secs(5 * 60),
            use_multiplexed_session: false,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("session get time out")]
    SessionGetTimeout,
    #[error("failed to create session")]
    FailedToCreateSession,
    #[error(transparent)]
    GRPC(#[from] Status),
}

impl TryAs<Status> for SessionError {
    fn try_as(&self) -> Option<&Status> {
        match self {
            SessionError::GRPC(e) => Some(e),
            _ => None,
        }
    }
}

/// Context required to (re)create a multiplexed session after the original
/// has been observed as `NotFound` server-side.
struct MultiplexedRecreateCtx {
    conn_pool: Arc<ConnectionManager>,
    database: String,
    disable_route_to_leader: bool,
    metrics: Arc<MetricsRecorder>,
}

pub(crate) struct SessionManager {
    session_pool: SessionPool,
    /// Multiplexed session mode: holds the single long-lived session used as a
    /// template. Each call to `get()` clones the session proto + gRPC client to
    /// produce a fresh `SessionHandle` without pooling or expiry semantics.
    multiplexed: RwLock<Option<SessionHandle>>,
    /// Shared with every multiplexed clone. A NotFound on any clone flips
    /// this to `true`; the next `get()` triggers a recreation. Subject to
    /// the generation check carried by `MultiplexedInvalidator` so stale
    /// clones cannot re-flag a freshly-recreated master.
    multiplexed_invalid: Arc<AtomicBool>,
    /// Bumped each time a new master multiplexed session is installed
    /// (initial create + every recreation). Each clone snapshots the
    /// current value at clone time; `invalidate_if_needed` only flips the
    /// invalid flag when the snapshot matches the current value.
    multiplexed_generation: Arc<AtomicU64>,
    /// Serializes concurrent recreation attempts (an async mutex because
    /// `create_multiplexed_session` is awaitable).
    multiplexed_create_lock: tokio::sync::Mutex<()>,
    multiplexed_recreate_ctx: Option<MultiplexedRecreateCtx>,
    cancel: CancellationToken,
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl SessionManager {
    pub async fn new(
        database: impl Into<String>,
        conn_pool: ConnectionManager,
        config: SessionConfig,
        disable_route_to_leader: bool,
        metrics: Arc<MetricsRecorder>,
    ) -> Result<Arc<SessionManager>, Status> {
        let database = database.into();
        let use_multiplexed = config.use_multiplexed_session;

        // For multiplexed mode we still create a minimal (empty) pool as a
        // placeholder — it satisfies the SessionPool type without pre-creating
        // any regular sessions (min_opened = 0).
        let pool_config = if use_multiplexed {
            let mut c = config.clone();
            c.min_opened = 0;
            c
        } else {
            config.clone()
        };

        let conn_pool = Arc::new(conn_pool);
        let (sender, receiver) = mpsc::unbounded_channel();
        let session_pool = SessionPool::new(
            database.clone(),
            conn_pool.as_ref(),
            sender,
            Arc::new(pool_config),
            disable_route_to_leader,
            metrics.clone(),
        )
        .await?;

        let cancel = CancellationToken::new();
        let multiplexed_invalid = Arc::new(AtomicBool::new(false));
        let multiplexed_generation = Arc::new(AtomicU64::new(0));

        let (multiplexed, tasks, multiplexed_recreate_ctx) = if use_multiplexed {
            // Create the single multiplexed session and store it.
            let handle =
                create_multiplexed_session(conn_pool.as_ref(), &database, disable_route_to_leader, metrics.clone())
                    .await?;
            tracing::debug!("multiplexed session created: {}", handle.session.name);
            // First master generation = 1 (clones default to 0; bumping here
            // ensures even the first clone snapshots a non-zero value that
            // matches a real installed master).
            multiplexed_generation.store(1, Ordering::Release);
            let ctx = MultiplexedRecreateCtx {
                conn_pool: conn_pool.clone(),
                database: database.clone(),
                disable_route_to_leader,
                metrics: metrics.clone(),
            };
            (RwLock::new(Some(handle)), Mutex::new(vec![]), Some(ctx))
        } else {
            // Standard pool mode: spin up the background maintenance tasks.
            let task_cleaner = Self::spawn_health_check_task(config, session_pool.clone(), cancel.clone());
            let task_creator = Self::spawn_session_creation_task(
                session_pool.clone(),
                database,
                conn_pool,
                receiver,
                cancel.clone(),
                disable_route_to_leader,
            );
            (RwLock::new(None), Mutex::new(vec![task_cleaner, task_creator]), None)
        };

        // Register the session-pool stats snapshot with the metrics recorder
        // now that we know whether a multiplexed session was created.
        let has_multiplexed = use_multiplexed && multiplexed.read().is_some();
        metrics.register_session_pool(session_pool.snapshot_fn(has_multiplexed));

        let sm = SessionManager {
            session_pool,
            multiplexed,
            multiplexed_invalid,
            multiplexed_generation,
            multiplexed_create_lock: tokio::sync::Mutex::new(()),
            multiplexed_recreate_ctx,
            cancel,
            tasks,
        };
        Ok(Arc::new(sm))
    }

    pub fn num_opened(&self) -> usize {
        self.session_pool.num_opened()
    }

    pub async fn get(&self) -> Result<ManagedSession, SessionError> {
        if self.cancel.is_cancelled() {
            return Err(SessionError::FailedToCreateSession);
        }
        if let Some(ctx) = &self.multiplexed_recreate_ctx {
            // Multiplexed mode: try the fast path; fall back to recreating
            // the master if a prior NotFound flagged it as invalid. Record
            // acquire/latency here so observability matches the pool path
            // (record_session_acquired/_acquire_latency are otherwise only
            // emitted from SessionPool::acquire).
            let started_at = Instant::now();
            let metrics = self.session_pool.metrics.clone();
            if !self.multiplexed_invalid.load(Ordering::Acquire) {
                if let Some(handle) = self.clone_multiplexed_handle() {
                    metrics.record_session_acquired();
                    metrics.record_session_acquire_latency(started_at.elapsed());
                    return Ok(handle);
                }
            }
            self.recreate_multiplexed_session(ctx).await?;
            let handle = self
                .clone_multiplexed_handle()
                .ok_or(SessionError::FailedToCreateSession)?;
            metrics.record_session_acquired();
            metrics.record_session_acquire_latency(started_at.elapsed());
            return Ok(handle);
        }
        self.session_pool.acquire().await
    }

    /// Clone session proto + gRPC client of the master multiplexed handle.
    /// The read guard is dropped at the end of the inner block, so the
    /// SessionHandle is constructed without the lock held. parking_lot's
    /// RwLock allows multiple concurrent readers, so this also lets parallel
    /// `get()` callers clone in parallel; only recreate_multiplexed_session
    /// needs the write lock.
    fn clone_multiplexed_handle(&self) -> Option<ManagedSession> {
        let snapshot = {
            let guard = self.multiplexed.read();
            guard.as_ref().map(|h| (h.session.clone(), h.spanner_client.clone()))
        };
        snapshot.map(|(session, client)| {
            let handle = SessionHandle::new_multiplexed_clone(
                session,
                client,
                Instant::now(),
                self.multiplexed_invalidator(),
            );
            ManagedSession::new_multiplexed(self.session_pool.clone(), handle)
        })
    }

    fn multiplexed_invalidator(&self) -> MultiplexedInvalidator {
        MultiplexedInvalidator {
            flag: self.multiplexed_invalid.clone(),
            clone_generation: self.multiplexed_generation.load(Ordering::Acquire),
            current_generation: self.multiplexed_generation.clone(),
        }
    }

    async fn recreate_multiplexed_session(&self, ctx: &MultiplexedRecreateCtx) -> Result<(), Status> {
        // Serialize concurrent recreation attempts; double-check the flag
        // after acquiring the lock so we only pay for one CreateSession RPC.
        let _guard = self.multiplexed_create_lock.lock().await;
        if self.cancel.is_cancelled() {
            return Err(Status::new(Code::Cancelled, "session manager is closed"));
        }
        if !self.multiplexed_invalid.load(Ordering::Acquire) && self.multiplexed.read().is_some() {
            return Ok(());
        }
        let new_handle = create_multiplexed_session(
            ctx.conn_pool.as_ref(),
            &ctx.database,
            ctx.disable_route_to_leader,
            ctx.metrics.clone(),
        )
        .await?;
        tracing::debug!("multiplexed session recreated: {}", new_handle.session.name);
        *self.multiplexed.write() = Some(new_handle);
        // Bump the generation BEFORE clearing the invalid flag: any
        // late-arriving NotFound from a stale clone will then observe
        // a mismatched generation and skip the flag flip.
        self.multiplexed_generation.fetch_add(1, Ordering::AcqRel);
        self.multiplexed_invalid.store(false, Ordering::Release);
        Ok(())
    }

    pub async fn close(&self) {
        if self.cancel.is_cancelled() {
            return;
        }
        self.cancel.cancel();
        let tasks = { mem::take(&mut *self.tasks.lock()) };
        for task in tasks {
            let _ = task.await;
        }
        // Drop the multiplexed master handle. Multiplexed sessions cannot
        // be deleted via the API per the v1 proto ("Multiplexed sessions
        // may not be deleted nor listed"); the server reclaims them via
        // its own TTL once the channel is gone.
        let _ = self.multiplexed.write().take();
        self.session_pool.close().await;
    }

    fn spawn_session_creation_task(
        session_pool: SessionPool,
        database: String,
        conn_pool: Arc<ConnectionManager>,
        mut rx: UnboundedReceiver<usize>,
        cancel: CancellationToken,
        disable_route_to_leader: bool,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut tasks = JoinSet::default();
            loop {
                select! {
                    biased;
                    _ = cancel.cancelled() => break,
                    Some(Ok((session_count, result))) = tasks.join_next(), if !tasks.is_empty() => {
                        session_pool.inner.write().replenish(session_count, result);
                    }
                    session_count = rx.recv() => match session_count {
                        Some(session_count) => {
                            let client = conn_pool
                                .conn()
                                .with_metrics(session_pool.metrics.clone())
                                .with_metadata(client_metadata(&database));
                            let database = database.clone();
                            tasks.spawn(async move { (session_count, batch_create_sessions(client, &database, session_count, disable_route_to_leader).await) });
                        },
                        None => continue
                    },
                }
            }
            tracing::trace!("shutdown session creation task.");
        })
    }

    fn spawn_health_check_task(
        config: SessionConfig,
        session_pool: SessionPool,
        cancel: CancellationToken,
    ) -> JoinHandle<()> {
        let start = Instant::now() + config.refresh_interval;
        let mut interval = tokio::time::interval_at(start.into(), config.refresh_interval);

        tokio::spawn(async move {
            loop {
                select! {
                    _ = interval.tick() => {},
                    _ = cancel.cancelled() => break
                }
                let now = Instant::now();

                // remove orphans first
                session_pool.remove_orphans().await;

                // start health check
                health_check(
                    now + Duration::from_nanos(1),
                    config.session_alive_trust_duration,
                    &session_pool,
                    cancel.clone(),
                )
                .await;
            }
            tracing::trace!("shutdown health check task.")
        })
    }
}

async fn health_check(
    now: Instant,
    session_alive_trust_duration: Duration,
    sessions: &SessionPool,
    cancel: CancellationToken,
) {
    tracing::trace!("start health check");
    let start = Instant::now();
    let sleep_duration = Duration::from_millis(10);
    loop {
        select! {
            _ = sleep(sleep_duration) => {},
            _ = cancel.cancelled() => break
        }
        let mut s = {
            // temporary take
            let mut locked = sessions.inner.write();
            match locked.take() {
                Some(mut s) => {
                    // all the session check complete.
                    if s.last_checked_at == now {
                        locked.release(s);
                        break;
                    }
                    if std::cmp::max(s.last_used_at, s.last_pong_at) + session_alive_trust_duration >= now {
                        s.last_checked_at = now;
                        locked.release(s);
                        continue;
                    }
                    s
                }
                None => break,
            }
        };

        let request = ping_query_request(s.session.name.clone());
        match s.spanner_client.execute_sql(request, true, None).await {
            Ok(_) => {
                s.last_checked_at = now;
                s.last_pong_at = now;
                sessions.recycle(s);
            }
            Err(_) => {
                s.delete().await;
                sessions.recycle(s);
            }
        }
    }
    tracing::trace!("end health check elapsed={}msec", start.elapsed().as_millis());
}

/// Create a single multiplexed session via the `CreateSession` RPC.
///
/// Multiplexed sessions are long-lived, shared across concurrent operations,
/// and do not expire. They are required by Spanner Omni (and supported by
/// Cloud Spanner for read-only workloads). See:
/// <https://cloud.google.com/spanner/docs/sessions#create_a_multiplexed_session>
async fn create_multiplexed_session(
    conn_pool: &ConnectionManager,
    database: &str,
    disable_route_to_leader: bool,
    metrics: Arc<MetricsRecorder>,
) -> Result<SessionHandle, Status> {
    let mut client = conn_pool
        .conn()
        .with_metrics(metrics)
        .with_metadata(client_metadata(database));
    let req = CreateSessionRequest {
        database: database.to_string(),
        session: Some(Session {
            multiplexed: true,
            ..Default::default()
        }),
    };
    let session = client
        .create_session(req, disable_route_to_leader, None)
        .await?
        .into_inner();
    Ok(SessionHandle::new(session, client, Instant::now()))
}

async fn batch_create_sessions(
    spanner_client: Client,
    database: &str,
    mut remaining_create_count: usize,
    disable_route_to_leader: bool,
) -> Result<Vec<SessionHandle>, Status> {
    let mut created = Vec::with_capacity(remaining_create_count);
    while remaining_create_count > 0 {
        let sessions = batch_create_session(
            spanner_client.clone(),
            database,
            remaining_create_count,
            disable_route_to_leader,
        )
        .await?;
        // Spanner could return less sessions than requested.
        // In that case, we should do another call using the same gRPC channel.
        let actually_created = sessions.len();
        remaining_create_count -= actually_created;
        created.extend(sessions);
    }
    Ok(created)
}

async fn batch_create_session(
    mut spanner_client: Client,
    database: &str,
    session_count: usize,
    disable_route_to_leader: bool,
) -> Result<Vec<SessionHandle>, Status> {
    let request = BatchCreateSessionsRequest {
        database: database.to_string(),
        session_template: None,
        session_count: session_count as i32,
    };

    tracing::debug!("spawn session creation request : session_count = {}", session_count);
    let response = spanner_client
        .batch_create_sessions(request, disable_route_to_leader, None)
        .await?
        .into_inner();

    let now = Instant::now();
    Ok(response
        .session
        .into_iter()
        .map(|s| SessionHandle::new(s, spanner_client.clone(), now))
        .collect::<Vec<SessionHandle>>())
}

pub(crate) fn client_metadata(database: &str) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    metadata.insert("google-cloud-resource-prefix", database.parse().unwrap());
    metadata
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use parking_lot::RwLock;
    use serial_test::serial;
    use tokio::time::sleep;
    use tokio_util::sync::CancellationToken;

    use google_cloud_gax::conn::{ConnectionOptions, Environment};
    use google_cloud_gax::grpc::{Code, Status};
    use google_cloud_googleapis::spanner::v1::ExecuteSqlRequest;

    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::metrics::MetricsRecorder;
    use crate::session::{
        batch_create_sessions, client_metadata, health_check, SessionConfig, SessionError, SessionManager,
    };

    pub const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

    #[ctor::ctor]
    fn init() {
        let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
            .add_directive("google_cloud_spanner=trace".parse().unwrap());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }

    async fn assert_rush(use_invalidate: bool, config: SessionConfig) -> Arc<SessionManager> {
        let cm = ConnectionManager::new(
            4,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let sm = SessionManager::new(DATABASE, cm, config, false, Arc::new(MetricsRecorder::default()))
            .await
            .unwrap();

        let counter = Arc::new(AtomicI64::new(0));
        let mut spawns = Vec::with_capacity(100);
        for _ in 0..100 {
            let sm = sm.clone();
            let counter = Arc::clone(&counter);
            spawns.push(tokio::spawn(async move {
                let mut session = sm.get().await.unwrap();
                if use_invalidate {
                    session.delete().await;
                }
                counter.fetch_add(1, Ordering::SeqCst);
                sleep(Duration::from_millis(300)).await;
            }));
        }
        for handler in spawns {
            let _ = handler.await;
        }
        sm
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_health_check_checked() {
        let cm = ConnectionManager::new(
            4,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let session_alive_trust_duration = Duration::from_millis(10);
        let config = SessionConfig {
            min_opened: 5,
            session_alive_trust_duration,
            max_opened: 5,
            ..Default::default()
        };
        let sm = std::sync::Arc::new(
            SessionManager::new(DATABASE, cm, config, false, Arc::new(MetricsRecorder::default()))
                .await
                .unwrap(),
        );
        sleep(Duration::from_secs(1)).await;

        let cancel = CancellationToken::new();
        health_check(Instant::now(), session_alive_trust_duration, &sm.session_pool, cancel.clone()).await;

        assert_eq!(sm.num_opened(), 5);
        tokio::time::sleep(Duration::from_millis(500)).await;
        cancel.cancel();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_health_check_not_checked() {
        let cm = ConnectionManager::new(
            4,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let session_alive_trust_duration = Duration::from_secs(10);
        let config = SessionConfig {
            min_opened: 5,
            session_alive_trust_duration,
            max_opened: 5,
            ..Default::default()
        };
        let sm = Arc::new(
            SessionManager::new(DATABASE, cm, config, false, Arc::new(MetricsRecorder::default()))
                .await
                .unwrap(),
        );
        sleep(Duration::from_secs(1)).await;

        let cancel = CancellationToken::new();
        health_check(Instant::now(), session_alive_trust_duration, &sm.session_pool, cancel.clone()).await;

        assert_eq!(sm.num_opened(), 5);
        sleep(Duration::from_millis(500)).await;
        cancel.cancel();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_increase_session_and_idle_session_expired() {
        let conn_pool = ConnectionManager::new(
            4,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let config = SessionConfig {
            idle_timeout: Duration::from_millis(10),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        let sm = SessionManager::new(DATABASE, conn_pool, config, false, Arc::new(MetricsRecorder::default()))
            .await
            .unwrap();
        {
            let mut sessions = Vec::new();
            for _ in 0..45 {
                sessions.push(sm.get().await.unwrap());
            }

            // all the session are using
            assert_eq!(sm.num_opened(), 45);
            assert_eq!(sm.session_pool.inner.read().num_inuse, 45, "all the session are using");
            sleep(Duration::from_secs(1)).await;
        }

        // idle session removed after drop
        let sessions = sm.session_pool.inner.read();
        assert_eq!(sessions.num_inuse, 0, "invalid num_inuse");
        assert_eq!(sessions.available_sessions.len(), 20, "invalid available sessions");
        assert_eq!(sessions.num_opened(), 20, "invalid num open");
        assert_eq!(sessions.waiters.len(), 0, "session waiters is 0");
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_too_many_session_timeout() {
        let conn_pool = ConnectionManager::new(
            4,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let config = SessionConfig {
            idle_timeout: Duration::from_millis(10),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            session_get_timeout: Duration::from_secs(1),
            ..Default::default()
        };
        let sm = Arc::new(
            SessionManager::new(DATABASE, conn_pool, config.clone(), false, Arc::new(MetricsRecorder::default()))
                .await
                .unwrap(),
        );
        let mu = Arc::new(RwLock::new(Vec::new()));
        let mut awaiters = Vec::with_capacity(100);
        for _ in 0..100 {
            let sm = sm.clone();
            let mu = mu.clone();
            awaiters.push(tokio::spawn(async move {
                let session = sm.get().await;
                mu.write().push(session);
                0
            }))
        }
        for handler in awaiters {
            let _ = handler.await;
        }
        let sessions = mu.read();
        for i in 0..sessions.len() - 1 {
            let session = &sessions[i];
            if i >= config.max_opened {
                assert!(session.is_err(), "must err {i}");
                match session.as_ref().err().unwrap() {
                    SessionError::SessionGetTimeout => {}
                    _ => {
                        panic!("must be session timeout error")
                    }
                }
            } else {
                assert!(session.is_ok(), "must ok {i}");
            }
        }
        let pool = sm.session_pool.inner.read();
        assert_eq!(pool.num_opened(), config.max_opened);
        assert_eq!(pool.waiters.len(), 100 - config.max_opened); //include timeout sessions
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_invalidate() {
        let config = SessionConfig {
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        let sm = assert_rush(true, config.clone()).await;
        {
            let sessions = sm.session_pool.inner.read();
            let available_sessions = sessions.available_sessions.len();
            assert_eq!(sessions.num_inuse, 0);
            assert_eq!(sessions.waiters.len(), 0);
            assert_eq!(sessions.orphans.len(), 0);
            assert!(
                available_sessions <= config.max_opened && available_sessions >= config.min_opened,
                "now is {available_sessions}"
            );
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush() {
        let config = SessionConfig {
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        let sm = assert_rush(false, config.clone()).await;
        {
            let sessions = sm.session_pool.inner.read();
            let available_sessions = sessions.available_sessions.len();
            assert_eq!(sessions.num_inuse, 0);
            assert_eq!(sessions.waiters.len(), 0);
            assert_eq!(sessions.orphans.len(), 0);
            assert!(
                available_sessions <= config.max_opened && available_sessions >= config.min_opened,
                "now is {available_sessions}"
            );
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_with_invalidate() {
        let config = SessionConfig {
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        let sm = assert_rush(true, config.clone()).await;
        {
            let sessions = sm.session_pool.inner.read();
            let available_sessions = sessions.available_sessions.len();
            assert_eq!(sessions.num_inuse, 0);
            assert_eq!(sessions.waiters.len(), 0);
            assert_eq!(sessions.orphans.len(), 0);
            assert!(
                available_sessions <= config.max_opened && available_sessions >= config.min_opened,
                "now is {available_sessions}"
            );
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_with_health_check() {
        let config = SessionConfig {
            session_alive_trust_duration: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(250),
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        let sm = assert_rush(false, config.clone()).await;
        sleep(Duration::from_secs(2)).await;
        {
            let sessions = sm.session_pool.inner.read();
            let available_sessions = sessions.available_sessions.len();
            assert!(sessions.num_inuse <= 1, "num_inuse is {}", sessions.num_inuse);
            assert_eq!(sessions.waiters.len(), 0);
            assert_eq!(sessions.orphans.len(), 0);
            assert!(
                available_sessions <= config.max_opened && available_sessions >= config.max_idle - 1,
                "now is {available_sessions}"
            );
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_with_health_check_and_invalidate() {
        let config = SessionConfig {
            session_alive_trust_duration: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(250),
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        let sm = assert_rush(true, config.clone()).await;
        sleep(Duration::from_secs(2)).await;
        {
            let sessions = sm.session_pool.inner.read();
            let available_sessions = sessions.available_sessions.len();
            assert!(sessions.num_inuse <= 1, "num_inuse is {}", sessions.num_inuse);
            assert_eq!(sessions.waiters.len(), 0);
            assert_eq!(sessions.orphans.len(), 0);
            assert!(
                available_sessions <= config.max_opened && available_sessions >= config.min_opened - 1,
                "now is {available_sessions}"
            );
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_with_idle_expired() {
        let config = SessionConfig {
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            idle_timeout: Duration::from_millis(1),
            ..Default::default()
        };
        let sm = assert_rush(false, config.clone()).await;
        {
            let sessions = sm.session_pool.inner.read();
            assert_eq!(sessions.num_inuse, 0);
            assert_eq!(sessions.waiters.len(), 0);
            assert_eq!(sessions.orphans.len(), config.max_opened - config.max_idle);
            assert_eq!(sessions.available_sessions.len(), config.max_idle);
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_with_health_check_and_idle_expired() {
        let config = SessionConfig {
            session_alive_trust_duration: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(250),
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            idle_timeout: Duration::from_millis(1),
            ..Default::default()
        };
        let sm = assert_rush(false, config.clone()).await;
        sleep(Duration::from_secs(1)).await;
        {
            let sessions = sm.session_pool.inner.read();
            assert!(sessions.num_inuse <= 1, "num_inuse is {}", sessions.num_inuse);
            assert_eq!(sessions.waiters.len(), 0);
            assert_eq!(sessions.orphans.len(), 0);
            let available_sessions = sessions.available_sessions.len();
            assert!(
                available_sessions >= config.min_opened - 1 && available_sessions <= config.max_idle,
                "now is {available_sessions}"
            );
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_with_health_check_and_idle_expired_and_invalid() {
        let config = SessionConfig {
            session_alive_trust_duration: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(250),
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            idle_timeout: Duration::from_millis(1),
            ..Default::default()
        };
        let sm = assert_rush(true, config.clone()).await;
        sleep(Duration::from_secs(2)).await;
        {
            let sessions = sm.session_pool.inner.read();
            assert!(sessions.num_inuse <= 1, "num_inuse is {}", sessions.num_inuse);
            // health checker removes orphans
            assert_eq!(sessions.orphans.len(), 0);
            assert_eq!(sessions.waiters.len(), 0, "invalid waiters");
            let available_sessions = sessions.available_sessions.len();
            assert!(
                available_sessions >= config.min_opened - 1 && available_sessions <= config.max_idle,
                "now is {available_sessions}"
            );
        }
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_close() {
        let cm = ConnectionManager::new(
            4,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let config = SessionConfig::default();
        let sm = SessionManager::new(DATABASE, cm, config.clone(), false, Arc::new(MetricsRecorder::default()))
            .await
            .unwrap();
        assert_eq!(sm.num_opened(), config.min_opened);
        sm.close().await;
        assert_eq!(sm.num_opened(), 0);
        assert_eq!(sm.session_pool.inner.read().orphans.len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multiplexed_session_recreated_after_invalidation() {
        let cm = ConnectionManager::new(
            1,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let config = SessionConfig {
            use_multiplexed_session: true,
            min_opened: 0,
            max_opened: 1,
            ..Default::default()
        };
        let sm = SessionManager::new(DATABASE, cm, config, false, Arc::new(MetricsRecorder::default()))
            .await
            .unwrap();

        // First get(): returns a clone of the freshly-created master.
        let s1 = sm.get().await.unwrap();
        let name1 = (*s1).session.name.clone();
        drop(s1);

        // Simulate what invalidate_if_needed would do on a NotFound from
        // any operation that uses the multiplexed session.
        sm.multiplexed_invalid.store(true, Ordering::Release);

        // Next get(): must recreate the master and hand back a clone of
        // the new session, which has a different server-assigned name.
        let s2 = sm.get().await.unwrap();
        let name2 = (*s2).session.name.clone();

        assert_ne!(name1, name2, "expected the master multiplexed session to be recreated");
        assert!(!sm.multiplexed_invalid.load(Ordering::Acquire), "invalid flag should be cleared after recreate");
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_stale_multiplexed_clone_does_not_reflag_invalid() {
        let cm = ConnectionManager::new(
            1,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let config = SessionConfig {
            use_multiplexed_session: true,
            min_opened: 0,
            max_opened: 1,
            ..Default::default()
        };
        let sm = SessionManager::new(DATABASE, cm, config, false, Arc::new(MetricsRecorder::default()))
            .await
            .unwrap();

        // Take a clone at generation N.
        let mut stale = sm.get().await.unwrap();

        // Simulate a recreation having already happened by bumping the
        // generation under the stale clone. Clear the invalid flag so we
        // can see whether `invalidate_if_needed` flips it back.
        sm.multiplexed_generation.fetch_add(1, Ordering::AcqRel);
        sm.multiplexed_invalid.store(false, Ordering::Release);

        // Mimic a server-side `NotFound` arriving on the stale clone. The
        // generation check must skip the flag flip.
        let err = Status::new(Code::NotFound, "Session not found: projects/.../sessions/stale");
        let _ = stale.invalidate_if_needed::<()>(Err(err)).await;
        assert!(
            !sm.multiplexed_invalid.load(Ordering::Acquire),
            "stale clone (older generation) must not re-flag the master invalid"
        );
        drop(stale);

        // A fresh clone snapshots the current generation; a NotFound on
        // it must flip the flag.
        let mut fresh = sm.get().await.unwrap();
        let err = Status::new(Code::NotFound, "Session not found: projects/.../sessions/fresh");
        let _ = fresh.invalidate_if_needed::<()>(Err(err)).await;
        assert!(
            sm.multiplexed_invalid.load(Ordering::Acquire),
            "current-generation clone must flip the master invalid flag"
        );
        drop(fresh);
        sm.close().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_get_after_close_returns_error() {
        let cm = ConnectionManager::new(
            4,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let config = SessionConfig::default();
        let sm = SessionManager::new(DATABASE, cm, config, false, Arc::new(MetricsRecorder::default()))
            .await
            .unwrap();
        sm.close().await;
        match sm.get().await {
            Err(SessionError::FailedToCreateSession) => {}
            Err(e) => panic!("expected FailedToCreateSession after close, got {e:?}"),
            Ok(_) => panic!("expected FailedToCreateSession after close, got Ok"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_batch_create_sessions() {
        let cm = ConnectionManager::new(
            1,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let client = cm
            .conn()
            .with_metrics(Arc::new(MetricsRecorder::default()))
            .with_metadata(client_metadata(DATABASE));
        let session_count = 125;
        let result = batch_create_sessions(client.clone(), DATABASE, session_count, false).await;
        match result {
            Ok(created) => {
                assert_eq!(session_count, created.len());
                for mut s in created {
                    let ping_result = s
                        .spanner_client
                        .execute_sql(
                            ExecuteSqlRequest {
                                session: s.session.name.to_string(),
                                transaction: None,
                                sql: "SELECT 1".to_string(),
                                params: None,
                                param_types: Default::default(),
                                resume_token: vec![],
                                query_mode: 0,
                                partition_token: vec![],
                                seqno: 0,
                                query_options: None,
                                request_options: None,
                                directed_read_options: None,
                                data_boost_enabled: false,
                                last_statement: false,
                            },
                            false,
                            None,
                        )
                        .await;
                    assert!(ping_result.is_ok());
                }
            }
            Err(err) => panic!("{err:?}"),
        }
    }
}

use std::collections::VecDeque;
use std::mem;
use std::ops::{Deref, DerefMut};
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

use google_cloud_gax::grpc::metadata::MetadataMap;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::TryAs;
use google_cloud_googleapis::spanner::v1::{BatchCreateSessionsRequest, DeleteSessionRequest, Session};

use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::spanner_client::{ping_query_request, Client};

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
        }
    }

    pub async fn invalidate_if_needed<T>(&mut self, arg: Result<T, Status>) -> Result<T, Status> {
        match arg {
            Ok(s) => Ok(s),
            Err(e) => {
                if e.code() == Code::NotFound && e.message().contains("Session not found:") {
                    tracing::debug!("session invalidate {}", self.session.name);
                    self.delete().await;
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
        match self.spanner_client.delete_session(request, None).await {
            Ok(_) => self.deleted = true,
            Err(e) => tracing::error!("failed to delete session {}, {:?}", session_name, e),
        };
    }
}

/// ManagedSession
pub struct ManagedSession {
    session_pool: SessionPool,
    session: Option<SessionHandle>,
}

impl ManagedSession {
    fn new(session_pool: SessionPool, session: SessionHandle) -> Self {
        ManagedSession {
            session_pool,
            session: Some(session),
        }
    }
}

impl Drop for ManagedSession {
    fn drop(&mut self) {
        let session = self.session.take().unwrap();
        self.session_pool.recycle(session);
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
                Some(s)
            }
        }
    }

    fn release(&mut self, session: SessionHandle) {
        self.num_inuse -= 1;
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
}

#[derive(Clone)]
struct SessionPool {
    inner: Arc<RwLock<Sessions>>,
    session_creation_sender: UnboundedSender<usize>,
    config: Arc<SessionConfig>,
}

impl SessionPool {
    async fn new(
        database: String,
        conn_pool: &ConnectionManager,
        session_creation_sender: UnboundedSender<usize>,
        config: Arc<SessionConfig>,
    ) -> Result<Self, Status> {
        let available_sessions = Self::init_pool(database, conn_pool, config.min_opened).await?;
        Ok(SessionPool {
            inner: Arc::new(RwLock::new(Sessions {
                available_sessions,
                waiters: VecDeque::new(),
                orphans: Vec::new(),
                num_inuse: 0,
                num_creating: 0,
            })),
            session_creation_sender,
            config,
        })
    }

    async fn init_pool(
        database: String,
        conn_pool: &ConnectionManager,
        min_opened: usize,
    ) -> Result<VecDeque<SessionHandle>, Status> {
        let channel_num = conn_pool.num();
        let creation_count_per_channel = min_opened / channel_num;
        let remainder = min_opened % channel_num;

        let mut sessions = Vec::<SessionHandle>::new();
        let mut tasks = JoinSet::new();
        for _ in 0..channel_num {
            // Ensure that we create the exact number of requested sessions by adding the remainder to the first channel.
            let creation_count = if channel_num == 0 {
                creation_count_per_channel + remainder
            } else {
                creation_count_per_channel
            };
            let next_client = conn_pool.conn().with_metadata(client_metadata(&database));
            let database = database.clone();
            tasks.spawn(async move { batch_create_sessions(next_client, &database, creation_count).await });
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
        loop {
            let (on_session_acquired, session_count) = {
                let mut sessions = self.inner.write();

                // Prioritize waiters over new acquirers.
                if sessions.waiters.is_empty() {
                    if let Some(mut s) = sessions.take() {
                        s.last_used_at = Instant::now();
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

pub(crate) struct SessionManager {
    session_pool: SessionPool,
    cancel: CancellationToken,
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl SessionManager {
    pub async fn new(
        database: impl Into<String>,
        conn_pool: ConnectionManager,
        config: SessionConfig,
    ) -> Result<Arc<SessionManager>, Status> {
        let database = database.into();
        let (sender, receiver) = mpsc::unbounded_channel();
        let session_pool = SessionPool::new(database.clone(), &conn_pool, sender, Arc::new(config.clone())).await?;

        let cancel = CancellationToken::new();
        let task_session_cleaner = Self::spawn_health_check_task(config, session_pool.clone(), cancel.clone());
        let task_session_creator =
            Self::spawn_session_creation_task(session_pool.clone(), database, conn_pool, receiver, cancel.clone());

        let sm = SessionManager {
            session_pool,
            cancel,
            tasks: Mutex::new(vec![task_session_cleaner, task_session_creator]),
        };
        Ok(Arc::new(sm))
    }

    pub fn num_opened(&self) -> usize {
        self.session_pool.num_opened()
    }

    pub async fn get(&self) -> Result<ManagedSession, SessionError> {
        self.session_pool.acquire().await
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
        self.session_pool.close().await;
    }

    fn spawn_session_creation_task(
        session_pool: SessionPool,
        database: String,
        conn_pool: ConnectionManager,
        mut rx: UnboundedReceiver<usize>,
        cancel: CancellationToken,
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
                            let client = conn_pool.conn().with_metadata(client_metadata(&database));
                            let database = database.clone();
                            tasks.spawn(async move { (session_count, batch_create_sessions(client, &database, session_count).await) });
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
        match s.spanner_client.execute_sql(request, None).await {
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

async fn batch_create_sessions(
    spanner_client: Client,
    database: &str,
    mut remaining_create_count: usize,
) -> Result<Vec<SessionHandle>, Status> {
    let mut created = Vec::with_capacity(remaining_create_count);
    while remaining_create_count > 0 {
        let sessions = batch_create_session(spanner_client.clone(), database, remaining_create_count).await?;
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
) -> Result<Vec<SessionHandle>, Status> {
    let request = BatchCreateSessionsRequest {
        database: database.to_string(),
        session_template: None,
        session_count: session_count as i32,
    };

    tracing::debug!("spawn session creation request : session_count = {}", session_count);
    let response = spanner_client.batch_create_sessions(request, None).await?.into_inner();

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
    use google_cloud_googleapis::spanner::v1::ExecuteSqlRequest;

    use crate::apiv1::conn_pool::ConnectionManager;
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
        let sm = SessionManager::new(DATABASE, cm, config).await.unwrap();

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
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
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
        let sm = Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
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
        let sm = SessionManager::new(DATABASE, conn_pool, config).await.unwrap();
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
        let sm = Arc::new(SessionManager::new(DATABASE, conn_pool, config.clone()).await.unwrap());
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
        let sm = SessionManager::new(DATABASE, cm, config.clone()).await.unwrap();
        assert_eq!(sm.num_opened(), config.min_opened);
        sm.close().await;
        assert_eq!(sm.num_opened(), 0);
        assert_eq!(sm.session_pool.inner.read().orphans.len(), 0);
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
        let client = cm.conn().with_metadata(client_metadata(DATABASE));
        let session_count = 125;
        let result = batch_create_sessions(client.clone(), DATABASE, session_count).await;
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

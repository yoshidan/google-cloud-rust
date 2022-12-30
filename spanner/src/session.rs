use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use thiserror;
use tokio::select;

use google_cloud_googleapis::spanner::v1::{BatchCreateSessionsRequest, DeleteSessionRequest, Session};

use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::spanner_client::{ping_query_request, Client};

use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::TryAs;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration};

type Waiters = Mutex<VecDeque<oneshot::Sender<SessionHandle>>>;

/// Session
pub struct SessionHandle {
    pub session: Session,
    pub spanner_client: Client,
    valid: bool,
    last_used_at: Instant,
    last_checked_at: Instant,
    last_pong_at: Instant,
    created_at: Instant
}

impl SessionHandle {
    pub(crate) fn new(session: Session, spanner_client: Client, now: Instant) -> SessionHandle {
        SessionHandle {
            session,
            spanner_client,
            valid: true,
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
        let session_name = &self.session.name;
        let request = DeleteSessionRequest {
            name: session_name.to_string(),
        };
        match self.spanner_client.delete_session(request, None, None).await {
            Ok(_s) => self.valid = false,
            Err(e) => tracing::error!("failed to delete session {}, {:?}", session_name, e),
        }
    }
}

/// ManagedSession
pub struct ManagedSession {
    session_pool: SessionPool,
    session: Option<SessionHandle>,
}

impl ManagedSession {
    pub(crate) fn new(session_pool: SessionPool, session: SessionHandle) -> Self {
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

pub struct SessionQueue {
    available_sessions: VecDeque<SessionHandle>,
    waiters: VecDeque<oneshot::Sender<SessionHandle>>,
    inuse: usize,
    allocating: usize
}

impl SessionQueue {

    fn grow(&mut self, mut new_sessions: Vec<SessionHandle>) {
        while let Some(session) = new_sessions.pop() {
            match self.waiters.pop_front() {
                Some(c) => {
                    match c.send(session) {
                        Err(session) => self.available_sessions.push_back(session),
                        _ => {
                            // Mark as using when notify to waiter directory.
                            self.inuse += 1
                        }
                    };
                }
                None => self.available_sessions.push_back(session)
            };
        }
    }

    fn num_opened(&self) -> usize {
        self.inuse + self.available_sessions.len()
    }

    fn take(&mut self) -> Option<SessionHandle> {
        match self.available_sessions.pop_front() {
            None => None,
            Some(s) => {
                self.inuse += 1;
                Some(s)
            }
        }
    }

    fn release(&mut self, session: SessionHandle) {
        self.inuse -= 1;
        if session.valid {
            self.available_sessions.push_back(session);
        }
    }
}

pub struct SessionPool {
    queue: Arc<Mutex<SessionQueue>>,
    allocation_request_sender: broadcast::Sender<usize>,
    config: Arc<SessionConfig>
}

impl SessionPool {
    async fn new(
        database: String,
        conn_pool: &ConnectionManager,
        allocation_request_sender: broadcast::Sender<usize>,
        config: Arc<SessionConfig>,
    ) -> Result<Self, Status> {
        let sessions = Self::init_pool(database, conn_pool, min_opened.clone()).await?;
        let waiters = Arc::new(VecDeque::new());

        Ok(SessionPool {
            queue: Arc::new(Mutex::new(Sessions {
                sessions,
                waiters,
                inuse: 0,
            })),
            allocation_request_sender,
            config
        })
    }

    async fn init_pool(
        database: String,
        conn_pool: &ConnectionManager,
        min_opened: usize,
    ) -> Result<VecDeque<SessionHandle>, Status> {
        let channel_num = conn_pool.num();
        let creation_count_per_channel = min_opened / channel_num;

        let mut sessions = Vec::<SessionHandle>::new();
        for _ in 0..channel_num {
            let next_client = conn_pool.conn();
            match batch_create_session(next_client, database.clone(), creation_count_per_channel).await {
                Ok(r) => {
                    for i in r {
                        sessions.push(i);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        tracing::debug!("initial session created count = {}", sessions.len());
        Ok(sessions.into())
    }

    fn acquire(&self, wait_timeout:Duration) -> Result<ManagedSession, SessionError> {
        let (on_session_acquired,session_allocation_count)= {
            let mut queue = self.queue.lock();

            // Prioritize waiters over new acquirers.
            if queue.waiters.is_empty() {
                if let Some(mut s) = queue.take() {
                    s.last_used_at = Instant::now();
                    return Ok(ManagedSession::new(self.clone(), s));
                }
            }
            // Add the participant to the waiting list.
            let (sender, receiver) = oneshot::channel();
            queue.waiters.push_back(sender);
            let session_allocation_count = self.get_session_allocation_count(&mut queue);
            (receiver, session_allocation_count)
        };

        if session_allocation_count > 0 {
           let _ = self.allocation_request_sender.send(session_allocation_count);
        }

        // Wait for the session available notification.
        match timeout(self.config.session_get_timeout, on_session_acquired).await {
            Ok(Ok(mut session)) => {
                session.last_used_at = Instant::now();
                Ok(ManagedSession {
                    session_pool: self.clone(),
                    session: Some(session),
                })
            }
            _ => Err(SessionError::SessionGetTimeout),
        }
    }

    fn num_opened(&self) -> usize {
        self.queue.lock().num_opened()
    }

    fn num_waiting(&self) -> usize {
        self.queue.lock().waiters.len()
    }

    fn recycle(&self, mut session: SessionHandle) {
        if session.valid {
            tracing::trace!("recycled name={}", session.session.name);
            let mut queue = self.queue.lock();
            match queue.waiters.pop_front() {
                // Immediately reuse session when the waiter exist
                Some(c) => {
                    if let Err(session) = c.send(session) {
                        queue.release(session)
                    }
                }
                None => {
                    if queue.num_opened() <= self.max_idle || session.created_at + self.idle_time > Instant::now() {
                        // Not reuse expired idle session
                        session.valid = false
                    }
                    queue.release(session)
                },
            };
        } else {
            let mut queue = self.queue.lock();
            queue.release(session);
            if queue.num_opened() < self.min_opened && !queue.waiters.is_empty() {
                let session_allocation_count = self.calculate_session_allocation_count(&mut queue);
                if session_allocation_count > 0 {
                   let _ = self.allocation_request_sender.send(session_allocation_count);
                }
            }
        }
    }

    fn calculate_session_allocation_count(&self, queue: &mut SessionQueue) -> usize {
        // Request allocation when no one request.
        let num_opened= queue.num_opened();
        let limit = self.max_open;
        let allocating = queue.allocating;
        let mut increasable = limit - (allocating + num_opened);
        if increasable <= 0 {
            tracing::trace!("No available connections max={}, allocating={}, current={}", limit, allocating, num_opened);
            return 0;
        }else if increasable > self.inc_step {
            return self.inc_step
        }
        return increasable;
    }

    async fn close(&self) {
        let deleting_sessions = {
            let mut queue= self.queue.lock();
            let mut deleting_sessions = Vec::with_capacity(queue.allocated.len());
            while let Some(session) = queue.available_sessions.pop_front() {
                deleting_sessions.push(session);
            }
            deleting_sessions
        };
        for mut session in deleting_sessions {
            session.delete().await;
        }
    }
}

impl Clone for SessionPool {
    fn clone(&self) -> Self {
        SessionPool {
            queue: Arc::clone(&self.queue),
            allocation_request_sender: self.allocation_request_sender.clone(),
            config: self.config.clone()
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
    pub idle_timeout: std::time::Duration,

    pub session_alive_trust_duration: std::time::Duration,

    /// session_get_timeout is the maximum value of the waiting time that occurs when retrieving from the connection pool when there is no idle session.
    pub session_get_timeout: std::time::Duration,

    /// refresh_interval is the interval of cleanup and health check functions.
    pub refresh_interval: std::time::Duration,

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
            idle_timeout: std::time::Duration::from_secs(30 * 60),
            session_alive_trust_duration: std::time::Duration::from_secs(55 * 60),
            session_get_timeout: std::time::Duration::from_secs(1),
            refresh_interval: std::time::Duration::from_secs(5 * 60),
        }
    }
}

pub struct SessionManager {
    session_pool: SessionPool,
    cancel: CancellationToken,
    tasks: Vec<JoinHandle<()>>,
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

impl SessionManager {
    pub async fn new(
        database: impl Into<String>,
        conn_pool: ConnectionManager,
        config: SessionConfig,
    ) -> Result<SessionManager, Status> {
        let database = database.into();
        let (sender, receiver) = broadcast::channel(1);
        let session_pool = SessionPool::new(database.clone(), &conn_pool, sender, Arc::new(config.clone())).await?;

        let cancel = CancellationToken::new();
        let task_cleaner = schedule_refresh(config, session_pool.clone(), cancel.clone());
        let task_listener = listen_session_creation_request(
            session_pool.clone(),
            database,
            conn_pool,
            receiver,
            cancel.clone(),
        );

        let sm = SessionManager {
            session_pool,
            cancel,
            tasks: vec![task_cleaner, task_listener],
        };
        Ok(sm)
    }

    pub fn num_opened(&self) -> usize {
        self.session_pool.num_opened()
    }

    pub fn session_waiters(&self) -> usize {
        self.session_pool.num_waiting()
    }

    pub async fn get(&self) -> Result<ManagedSession, SessionError> {
        self.session_pool.acquire(self.session_get_timeout)
    }

    pub(crate) async fn close(&self) {
        if self.cancel.is_cancelled() {
            return;
        }
        self.cancel.cancel();
        sleep(Duration::from_secs(1)).await;
        for task in &self.tasks {
            task.abort();
        }
        self.session_pool.close().await;
    }
}

fn listen_session_creation_request(
    session_pool: SessionPool,
    database: String,
    conn_pool: ConnectionManager,
    mut rx: broadcast::Receiver<usize>,
    cancel: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let creation_count = select! {
                _ = rx.recv(count) => count,
                _ = cancel.cancelled() => break
            };
            let database = database.clone();
            let next_client = conn_pool.conn();

            match batch_create_session(next_client, database, creation_count).await {
                Ok(fresh_sessions) => {
                    let mut queue = session_pool.queue.lock();
                    queue.allocating -= creation_count;
                    queue.grow(fresh_sessions)
                }
                Err(e) => {
                    let queue = session_pool.queue.lock();
                    queue.allocating -= creation_count;
                    tracing::error!("failed to create new sessions {:?}", e)
                }
            };
        }
        tracing::trace!("stop session creating listener")
    })
}

fn schedule_refresh(config: SessionConfig, session_pool: SessionPool, cancel: CancellationToken) -> JoinHandle<()> {
    let start = Instant::now() + config.refresh_interval;
    let mut interval = tokio::time::interval_at(start.into(), config.refresh_interval);

    tokio::spawn(async move {
        loop {
            select! {
                _ = interval.tick() => {},
                _ = cancel.cancelled() => break
            }
            let now = Instant::now();
            health_check(
                now + Duration::from_nanos(1),
                config.session_alive_trust_duration,
                &session_pool,
                cancel.clone(),
            )
            .await;
        }
        tracing::trace!("stop session cleaner")
    })
}

async fn health_check(
    now: Instant,
    session_alive_trust_duration: Duration,
    sessions: &SessionPool,
    cancel: CancellationToken,
) {
    let sleep_duration = Duration::from_millis(10);
    loop {
        select! {
            _ = sleep(sleep_duration) => {},
            _ = cancel.cancelled() => break
        }
        let mut s = {
            // temporary take
            let mut locked = sessions.queue.lock();
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
        match s.spanner_client.execute_sql(request, None, None).await {
            Ok(_) => {
                s.last_checked_at = now;
                s.last_pong_at = now;
                sessions.recycle(s);
            }
            Err(_) => {
                s.delete().await;
                s.valid = false;
                sessions.recycle(s);
            }
        }
    }
}

async fn batch_create_session(
    mut spanner_client: Client,
    database: String,
    creation_count: usize,
) -> Result<Vec<SessionHandle>, Status> {
    let request = BatchCreateSessionsRequest {
        database,
        session_template: None,
        session_count: creation_count as i32,
    };

    tracing::debug!("spawn session creation request : count to create = {}", creation_count);
    let response = spanner_client
        .batch_create_sessions(request, None, None)
        .await?
        .into_inner();

    let now = Instant::now();
    Ok(response
        .session
        .into_iter()
        .map(|s| SessionHandle::new(s, spanner_client.clone(), now))
        .collect::<Vec<SessionHandle>>())
}

#[cfg(test)]
mod tests {
    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::session::{health_check, shrink_idle_sessions, SessionConfig, SessionManager};
    use serial_test::serial;

    use google_cloud_gax::cancel::CancellationToken;
    use google_cloud_gax::conn::Environment;
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::time::{sleep, Duration};

    pub const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

    async fn assert_rush(use_invalidate: bool, config: SessionConfig) {
        let cm = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
            .await
            .unwrap();
        let max = config.max_opened;
        let min = config.min_opened;
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());

        let counter = Arc::new(AtomicI64::new(0));
        for _ in 0..100 {
            let sm = sm.clone();
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                let mut session = sm.get().await.unwrap();
                if use_invalidate {
                    session.delete().await;
                }
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }
        while counter.load(Ordering::SeqCst) < 100 {
            sleep(Duration::from_millis(5)).await;
        }

        sleep(tokio::time::Duration::from_millis(100)).await;
        assert_eq!(sm.session_pool.inner.lock().inuse, 0);
        let num_opened = sm.num_opened();
        assert!(num_opened <= max, "idle session must be lteq {} now is {}", max, num_opened);
        assert!(num_opened >= min, "idle session must be gteq {} now is {}", min, num_opened);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_shrink_sessions_not_expired() {
        let cm = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
            .await
            .unwrap();
        let idle_timeout = Duration::from_secs(100);
        let config = SessionConfig {
            min_opened: 5,
            idle_timeout,
            max_opened: 5,
            ..Default::default()
        };
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
        sleep(Duration::from_secs(1)).await;

        let cancel = CancellationToken::new();
        shrink_idle_sessions(Instant::now(), idle_timeout, &sm.session_pool, 5, cancel.clone()).await;

        assert_eq!(sm.num_opened(), 5);
        cancel.cancel();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_shrink_sessions_all_expired() {
        let cm = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
            .await
            .unwrap();
        let idle_timeout = Duration::from_millis(1);
        let config = SessionConfig {
            min_opened: 5,
            idle_timeout,
            max_opened: 5,
            ..Default::default()
        };
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
        sleep(Duration::from_secs(1)).await;
        let cancel = CancellationToken::new();
        shrink_idle_sessions(Instant::now(), idle_timeout, &sm.session_pool, 100, cancel.clone()).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        cancel.cancel();

        // expired but created by allocation batch
        assert_eq!(sm.num_opened(), 5);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_health_check_checked() {
        let cm = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
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
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        cancel.cancel();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_health_check_not_checked() {
        let cm = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
            .await
            .unwrap();
        let session_alive_trust_duration = Duration::from_secs(10);
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
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        cancel.cancel();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_schedule_refresh() {
        let conn_pool = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
            .await
            .unwrap();
        let config = SessionConfig {
            idle_timeout: Duration::from_millis(10),
            session_alive_trust_duration: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(250),
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
            {
                assert_eq!(sm.session_pool.inner.lock().inuse, 45, "all the session are using");
            }
            sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // idle session removed after cleanup
        sleep(tokio::time::Duration::from_secs(3)).await;
        {
            let available_sessions = sm.session_pool.inner.lock().sessions.len();
            assert!(
                available_sessions == 19 || available_sessions == 20,
                "available sessions are 19 or 20 (19 means that the cleaner is popping session)"
            );
        }
        assert_eq!(sm.num_opened(), 20, "num sessions are 20");
        assert_eq!(sm.session_waiters(), 0, "session waiters is 0");
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
        assert_rush(true, config).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_invalidate_with_cleanup() {
        let config = SessionConfig {
            idle_timeout: Duration::from_millis(10),
            session_alive_trust_duration: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(250),
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        assert_rush(true, config).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush() {
        let config = SessionConfig {
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        assert_rush(false, config).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_rush_with_cleanup() {
        let config = SessionConfig {
            idle_timeout: Duration::from_millis(10),
            session_alive_trust_duration: Duration::from_millis(10),
            refresh_interval: Duration::from_millis(250),
            session_get_timeout: Duration::from_secs(20),
            min_opened: 10,
            max_idle: 20,
            max_opened: 45,
            ..Default::default()
        };
        assert_rush(false, config).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_close() {
        let cm = ConnectionManager::new(1, &Environment::Emulator("localhost:9010".to_string()), "")
            .await
            .unwrap();
        let config = SessionConfig::default();
        let sm = SessionManager::new(DATABASE, cm, config.clone()).await.unwrap();
        assert_eq!(sm.num_opened(), config.min_opened);
        sm.close().await;
        assert_eq!(sm.num_opened(), 0)
    }
}

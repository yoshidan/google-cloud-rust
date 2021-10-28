use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Weak};
use std::time::Instant;

use parking_lot::Mutex;
use thiserror;
use tonic::metadata::KeyAndValueRef;
use tonic::Code;
use tonic::Status;

use google_cloud_googleapis::spanner::v1::{
    BatchCreateSessionsRequest, DeleteSessionRequest, Session,
};

use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::spanner_client::{ping_query_request, Client};
use std::sync::atomic::Ordering::SeqCst;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::time::{sleep, timeout, Duration};

type Waiters = Mutex<VecDeque<oneshot::Sender<SessionHandle>>>;

/// Session
pub struct SessionHandle {
    pub session: Session,
    pub spanner_client: Client,
    valid: bool,
    last_used_at: std::time::Instant,
    last_checked_at: std::time::Instant,
    last_pong_at: std::time::Instant,
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
        }
    }

    pub async fn invalidate_if_needed<T>(&mut self, arg: Result<T, Status>) -> Result<T, Status> {
        return match arg {
            Ok(s) => Ok(s),
            Err(e) => {
                if e.code() == Code::NotFound {
                    self.invalidate().await;
                }
                Err(e)
            }
        };
    }

    async fn invalidate(&mut self) {
        log::debug!("session invalidate {}", self.session.name);
        let request = DeleteSessionRequest {
            name: self.session.name.to_string(),
        };
        match self.spanner_client.delete_session(request, None).await {
            Ok(_s) => self.valid = false,
            Err(e) => {
                log::error!("session remove error {} error={:?}", self.session.name, e);
            }
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
        &self.session.as_ref().unwrap()
    }
}

impl DerefMut for ManagedSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.session.as_mut().unwrap()
    }
}

pub struct Sessions {
    sessions: VecDeque<SessionHandle>,
    inuse: usize,
}

impl Sessions {
    fn grow(&mut self, session: SessionHandle) {
        self.sessions.push_back(session);
    }

    fn notify_discarded(&mut self) {
        self.inuse -= 1;
    }

    fn num_opened(&self) -> usize {
        self.inuse + self.sessions.len()
    }

    fn take(&mut self) -> Option<SessionHandle> {
        match self.sessions.pop_front() {
            None => None,
            Some(s) => {
                self.inuse += 1;
                Some(s)
            }
        }
    }

    fn release(&mut self, session: SessionHandle) {
        self.inuse -= 1;
        self.sessions.push_back(session)
    }
}

pub struct SessionPool {
    inner: Arc<Mutex<Sessions>>,
    waiters: Arc<Waiters>,
    creation_producer: broadcast::Sender<bool>,
}

impl SessionPool {
    fn new(
        init_pool: VecDeque<SessionHandle>,
        waiters: Arc<Waiters>,
        creation_producer: broadcast::Sender<bool>,
    ) -> Self {
        SessionPool {
            inner: Arc::new(Mutex::new(Sessions {
                sessions: init_pool,
                inuse: 0,
            })),
            waiters,
            creation_producer,
        }
    }

    fn num_opened(&self) -> usize {
        self.inner.lock().num_opened()
    }

    fn grow(&self, mut sessions: Vec<SessionHandle>) {
        while let Some(session) = sessions.pop() {
            match { self.waiters.lock().pop_front() } {
                Some(c) => {
                    let mut inner = self.inner.lock();
                    match c.send(session) {
                        Err(session) => inner.grow(session),
                        _ => inner.inuse += 1,
                    };
                }
                None => self.inner.lock().grow(session),
            };
        }
    }

    fn recycle(&self, session: SessionHandle) {
        if session.valid {
            match { self.waiters.lock().pop_front() } {
                Some(c) => {
                    if let Err(session) = c.send(session) {
                        self.inner.lock().release(session)
                    }
                }
                None => self.inner.lock().release(session),
            };
        } else {
            {
                self.inner.lock().notify_discarded()
            };

            // request session creation
            self.creation_producer.send(true);
        }
    }
}

impl Clone for SessionPool {
    fn clone(&self) -> Self {
        SessionPool {
            inner: Arc::clone(&self.inner),
            waiters: Arc::clone(&self.waiters),
            creation_producer: self.creation_producer.clone(),
        }
    }
}

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
    database: String,
    conn_pool: Arc<ConnectionManager>,
    session_pool: SessionPool,
    config: Arc<SessionConfig>,
    waiters: Arc<Waiters>,
    creation_producer: broadcast::Sender<bool>,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("session get time out")]
    SessionGetTimeout,
    #[error("failed to create session")]
    FailedToCreateSession,
    #[error(transparent)]
    TonicError(#[from] Status),
}

impl SessionManager {
    pub async fn new(
        database: impl Into<String>,
        conn_pool: ConnectionManager,
        config: SessionConfig,
    ) -> Result<SessionManager, Status> {
        let database_name = database.into();
        let init_pool =
            SessionManager::init_pool(database_name.clone(), &conn_pool, config.min_opened).await?;

        let waiters = Arc::new(Waiters::new(VecDeque::new()));
        let (creation_producer, creation_consumer) = broadcast::channel(1);

        let sm = SessionManager {
            database: database_name,
            config: Arc::new(config),
            conn_pool: Arc::new(conn_pool),
            session_pool: SessionPool::new(
                init_pool,
                Arc::clone(&waiters),
                creation_producer.clone(),
            ),
            waiters,
            creation_producer,
        };

        // wait for batch creation request
        sm.listen_session_creation_request(creation_consumer);

        Ok(sm)
    }

    pub fn idle_sessions(&self) -> usize {
        self.session_pool.num_opened()
    }

    pub fn session_waiters(&self) -> usize {
        self.waiters.lock().len()
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
            match batch_create_session(next_client, database.clone(), creation_count_per_channel)
                .await
            {
                Ok(r) => {
                    for i in r {
                        sessions.push(i);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        log::debug!("initial session created count = {}", sessions.len());
        Ok(sessions.into())
    }

    pub async fn get(&self) -> Result<ManagedSession, SessionError> {
        if let Some(mut s) = self.session_pool.inner.lock().take() {
            s.last_used_at = Instant::now();
            return Ok(ManagedSession::new(self.session_pool.clone(), s));
        }

        let (sender, receiver) = oneshot::channel();
        {
            self.waiters.lock().push_back(sender);
        }

        // Request for creating batch.
        self.creation_producer.send(true);

        // Wait for the session creation.
        return match timeout(self.config.session_get_timeout, receiver).await {
            Ok(Ok(mut session)) => {
                session.last_used_at = Instant::now();
                Ok(ManagedSession {
                    session_pool: self.session_pool.clone(),
                    session: Some(session),
                })
            }
            _ => Err(SessionError::SessionGetTimeout),
        };
    }

    fn listen_session_creation_request(&self, mut rx: broadcast::Receiver<bool>) {
        let config = Arc::clone(&self.config);
        let session_pool = self.session_pool.clone();
        let database = self.database.clone();
        let conn_pool = Arc::clone(&self.conn_pool);
        tokio::spawn(async move {
            let mut allocation_request_size = 0;
            loop {
                let _ = rx.recv().await;

                let num_opened = session_pool.num_opened();
                if num_opened >= config.min_opened
                    && allocation_request_size >= { session_pool.waiters.lock().len() }
                {
                    continue;
                }

                let mut creation_count = config.max_opened - num_opened;
                if creation_count > config.inc_step {
                    creation_count = config.inc_step;
                }
                if creation_count == 0 {
                    println!("maximum session opened");
                    continue;
                }
                allocation_request_size += creation_count;

                let database = database.clone();
                let next_client = conn_pool.conn();
                println!("start batch create session {}", creation_count);

                match batch_create_session(next_client, database, creation_count).await {
                    Ok(fresh_sessions) => {
                        allocation_request_size -= creation_count;
                        session_pool.grow(fresh_sessions)
                    }
                    Err(e) => log::error!("failed to batch creation request {:?}", e),
                };
            }
        });
    }

    pub(crate) async fn close(&self) {
        let mut sessions = self.session_pool.inner.lock();
        while let Some(mut session) = sessions.take() {
            delete_session(&mut session).await;
            sessions.notify_discarded();
        }
    }

    pub(crate) fn schedule_refresh(&self) {
        let config = Arc::clone(&self.config);
        let start = Instant::now() + config.refresh_interval;
        let mut interval = tokio::time::interval_at(start.into(), config.refresh_interval);
        let session_pool = self.session_pool.clone();

        tokio::spawn(async move {
            loop {
                let _ = interval.tick().await;

                let max_removing_count = session_pool.num_opened() - config.max_idle;
                if max_removing_count < 0 {
                    continue;
                }

                let now = Instant::now();
                shrink_idle_sessions(now, config.idle_timeout, &session_pool, max_removing_count)
                    .await;
                health_check(
                    now + Duration::from_nanos(1),
                    config.session_alive_trust_duration,
                    &session_pool,
                )
                .await;
            }
        });
    }
}

async fn health_check(
    now: Instant,
    session_alive_trust_duration: Duration,
    sessions: &SessionPool,
) {
    let sleep_duration = Duration::from_millis(10);
    loop {
        sleep(sleep_duration).await;

        let mut s = {
            // temporary take
            let mut locked = sessions.inner.lock();
            match locked.take() {
                Some(mut s) => {
                    // all the session check complete.
                    if s.last_checked_at == now {
                        locked.release(s);
                        break;
                    }
                    if std::cmp::max(s.last_used_at, s.last_pong_at) + session_alive_trust_duration
                        >= now
                    {
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
                s.last_checked_at = now.clone();
                s.last_pong_at = now;
                sessions.recycle(s);
            }
            Err(err) => {
                log::error!("ping session err {:?}", err);
                delete_session(&mut s).await;
                s.valid = false;
                sessions.recycle(s);
            }
        }
    }
}

async fn shrink_idle_sessions(
    now: Instant,
    idle_timeout: Duration,
    session_pool: &SessionPool,
    max_shrink_count: usize,
) {
    let mut removed_count = 0;
    let sleep_duration = Duration::from_millis(10);
    loop {
        if removed_count >= max_shrink_count {
            break;
        }

        sleep(sleep_duration).await;

        // get old session
        let mut s = {
            // temporary take
            let mut locked = session_pool.inner.lock();
            match locked.take() {
                Some(mut s) => {
                    // all the session check complete.
                    if s.last_checked_at == now {
                        locked.release(s);
                        break;
                    }
                    if s.last_used_at + idle_timeout >= now {
                        s.last_checked_at = now;
                        locked.release(s);
                        continue;
                    }
                    s
                }
                None => break,
            }
        };

        removed_count += 1;
        delete_session(&mut s).await;
        s.valid = false;
        session_pool.recycle(s);
    }
}

async fn delete_session(session: &mut SessionHandle) {
    let session_name = &session.session.name;
    log::info!("delete session {}", session_name);
    let request = DeleteSessionRequest {
        name: session_name.to_string(),
    };
    match session.spanner_client.delete_session(request, None).await {
        Ok(_) => {}
        Err(e) => log::error!("failed to delete session {}, {:?}", session_name, e),
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
        session_count: creation_count.clone() as i32,
    };

    let response = spanner_client
        .batch_create_sessions(request, None)
        .await?
        .into_inner();
    log::info!("batch session created {}", creation_count);

    let now = Instant::now();
    return Ok(response
        .session
        .into_iter()
        .map(|s| SessionHandle::new(s, spanner_client.clone(), now))
        .collect::<Vec<SessionHandle>>());
}

#[cfg(test)]
mod tests {
    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::session_pool::{health_check, shrink_idle_sessions, SessionConfig, SessionManager};
    use serial_test::serial;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::time::{sleep, Duration};

    pub const DATABASE: &str =
        "projects/local-project/instances/test-instance/databases/local-database";

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_shrink_sessions_not_expired() {
        let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
            .await
            .unwrap();
        let idle_timeout = Duration::from_secs(100);
        let mut config = SessionConfig::default();
        config.min_opened = 5;
        config.idle_timeout = idle_timeout.clone();
        config.max_opened = 5;
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
        sleep(Duration::from_secs(1)).await;
        shrink_idle_sessions(Instant::now(), idle_timeout, &sm.session_pool, 5).await;

        assert_eq!(sm.idle_sessions(), 5);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_shrink_sessions_all_expired() {
        let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
            .await
            .unwrap();
        let idle_timeout = Duration::from_millis(1);
        let mut config = SessionConfig::default();
        config.min_opened = 5;
        config.idle_timeout = idle_timeout.clone();
        config.max_opened = 5;
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
        sleep(Duration::from_secs(1)).await;
        shrink_idle_sessions(Instant::now(), idle_timeout, &sm.session_pool, 100).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        // expired but created by allocation batch
        assert_eq!(sm.idle_sessions(), 5);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_health_check_checked() {
        let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
            .await
            .unwrap();
        let session_alive_trust_duration = Duration::from_millis(10);
        let mut config = SessionConfig::default();
        config.min_opened = 5;
        config.session_alive_trust_duration = session_alive_trust_duration.clone();
        config.max_opened = 5;
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
        sleep(Duration::from_secs(1)).await;

        health_check(
            Instant::now(),
            session_alive_trust_duration,
            &sm.session_pool,
        )
        .await;

        assert_eq!(sm.idle_sessions(), 5);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_health_check_not_checked() {
        let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
            .await
            .unwrap();
        let session_alive_trust_duration = Duration::from_secs(10);
        let mut config = SessionConfig::default();
        config.min_opened = 5;
        config.session_alive_trust_duration = session_alive_trust_duration.clone();
        config.max_opened = 5;
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
        sleep(Duration::from_secs(1)).await;

        health_check(
            Instant::now(),
            session_alive_trust_duration,
            &sm.session_pool,
        )
        .await;

        assert_eq!(sm.idle_sessions(), 5);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_schedule_refresh() {
        let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
            .await
            .unwrap();
        let mut config = SessionConfig::default();
        config.idle_timeout = Duration::from_millis(10);
        config.session_alive_trust_duration = Duration::from_millis(10);
        config.refresh_interval = Duration::from_millis(250);
        config.min_opened = 10;
        config.max_idle = 20;
        config.max_opened = 45;
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
        sm.schedule_refresh();

        {
            let mut sessions = Vec::new();
            for _ in 0..45 {
                sessions.push(sm.get().await.unwrap());
            }

            // all the session are using
            assert_eq!(sm.idle_sessions(), 45);
            {
                assert_eq!(
                    sm.session_pool.inner.lock().inuse,
                    45,
                    "all the session are using"
                );
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
        assert_eq!(sm.idle_sessions(), 20, "num sessions are 20");
        assert_eq!(sm.session_waiters(), 0, "session waiters is 0");
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_invalidate() {
        let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
            .await
            .unwrap();
        let mut config = SessionConfig::default();
        config.session_get_timeout = Duration::from_secs(20);
        config.min_opened = 10;
        config.max_idle = 20;
        config.max_opened = 45;
        let max = config.max_opened.clone();
        let min = config.min_opened.clone();
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());

        let counter = Arc::new(AtomicI64::new(0));
        for _ in 0..100 {
            let sm = sm.clone();
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                let mut session = sm.get().await.unwrap();
                session.invalidate().await;
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }
        while counter.load(Ordering::SeqCst) < 100 {
            sleep(Duration::from_millis(5)).await;
        }

        assert_eq!(sm.session_pool.inner.lock().inuse, 0);
        assert!(
            sm.idle_sessions() <= max,
            "idle session must be lteq {}",
            max
        );
        assert!(
            sm.idle_sessions() >= min,
            "idle session must be gteq {}",
            min
        );
    }
}

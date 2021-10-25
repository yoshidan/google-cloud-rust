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
use tokio::time::{sleep, Duration};

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
        if session.valid {
            self.session_pool.sessions.lock().push_back(session);
            self.session_pool.session_waiting_channel.send(true);
        } else {
            self.session_pool.num_opened.fetch_add(-1, Ordering::SeqCst);
        }
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

pub struct SessionPool {
    sessions: Arc<Mutex<VecDeque<SessionHandle>>>,
    num_opened: Arc<AtomicI64>,
    session_waiting_channel: Arc<tokio::sync::broadcast::Sender<bool>>,
}

impl SessionPool {
    fn new(
        init_pool: VecDeque<SessionHandle>,
        session_waiting_channel: Arc<tokio::sync::broadcast::Sender<bool>>,
    ) -> Self {
        let size = init_pool.len() as i64;
        SessionPool {
            sessions: Arc::new(Mutex::new(init_pool)),
            num_opened: Arc::new(AtomicI64::new(size)),
            session_waiting_channel,
        }
    }
}

impl Clone for SessionPool {
    fn clone(&self) -> Self {
        SessionPool {
            sessions: Arc::clone(&self.sessions),
            num_opened: Arc::clone(&self.num_opened),
            session_waiting_channel: Arc::clone(&self.session_waiting_channel),
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
    conn_pool: ConnectionManager,
    session_pool: SessionPool,
    config: SessionConfig,
    session_waiting_channel: Arc<tokio::sync::broadcast::Sender<bool>>,
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

        let (tx, _) = tokio::sync::broadcast::channel(100);
        let arc_tx = Arc::new(tx);
        return Ok(SessionManager {
            database: database_name,
            config,
            conn_pool,
            session_pool: SessionPool::new(init_pool, Arc::clone(&arc_tx)),
            session_waiting_channel: arc_tx,
        });
    }

    pub fn idle_sessions(&self) -> i64 {
        self.session_pool.num_opened.load(Ordering::SeqCst)
    }

    pub fn session_waiters(&self) -> usize {
        self.session_waiting_channel.receiver_count()
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
        loop {
            {
                let mut locked = self.session_pool.sessions.lock();
                while let Some(mut s) = locked.pop_front() {
                    s.last_used_at = Instant::now();
                    //Found valid session
                    return Ok(ManagedSession::new(self.session_pool.clone(), s));
                }
            };

            // Start to create session if not scheduled.
            if self.session_waiting_channel.receiver_count() == 0 {
                self.schedule_batch_create();
            }

            // Wait for session available.
            let (cancel_producer, cancel_consumer) = tokio::sync::oneshot::channel::<bool>();
            let session_get_timeout = self.config.session_get_timeout;
            tokio::spawn(async move {
                tokio::time::sleep(session_get_timeout).await;
                cancel_producer.send(true);
            });
            let mut rx = self.session_waiting_channel.subscribe();
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(s) => {
                            if !s {
                                return Err(SessionError::FailedToCreateSession);
                            }
                        },
                        Err(e) => {
                            log::error!("session creation failure : {:?}", e);
                            return Err(SessionError::FailedToCreateSession)
                        }
                    }
                }
                _ = cancel_consumer => return Err(SessionError::SessionGetTimeout),
            };
        }
    }

    fn schedule_batch_create(&self) {
        let mut creation_count =
            self.config.max_opened - self.session_pool.num_opened.load(Ordering::SeqCst) as usize;
        if creation_count > self.config.inc_step {
            creation_count = self.config.inc_step;
        }

        let database = self.database.clone();
        let idle_sessions = Arc::downgrade(&self.session_pool.sessions);
        let next_client = self.conn_pool.conn();
        let session_waiting_channel = Arc::clone(&self.session_waiting_channel);
        let num_opened = Arc::clone(&self.session_pool.num_opened);
        if creation_count == 0 {
            log::warn!("maximum session opened");
            return;
        }

        tokio::spawn(async move {
            log::debug!("start batch create session {}", creation_count);
            let result = match batch_create_session(next_client, database, creation_count).await {
                Ok(mut fresh_sessions) => {
                    // Register fresh sessions into pool.
                    match idle_sessions.upgrade() {
                        Some(g) => {
                            let mut locked_idle_session = g.lock();
                            while let Some(session) = fresh_sessions.pop() {
                                locked_idle_session.push_back(session);
                            }
                            //Update idle session cound
                            num_opened.fetch_add(creation_count as i64, Ordering::SeqCst);
                            true
                        }
                        None => {
                            log::error!("idle session pool already released.");
                            false
                        }
                    }
                }
                Err(e) => {
                    log::error!("failed to batch creation request {:?}", e);
                    false
                }
            };
            match session_waiting_channel.send(result) {
                Ok(s) => log::trace!("notified to {} receiver", s),
                Err(e) => log::error!("failed to notify session created {:?}", e),
            }
        });
    }

    pub(crate) async fn close(&self) {
        let mut sessions = self.session_pool.sessions.lock();
        while let Some(session) = sessions.pop_front() {
            delete_session(session).await;
        }
    }

    pub(crate) fn schedule_refresh(&self) {
        let max_idle = self.config.max_idle;
        let start = Instant::now() + self.config.refresh_interval;
        let mut interval = tokio::time::interval_at(start.into(), self.config.refresh_interval);
        let num_opened = Arc::clone(&self.session_pool.num_opened);
        let sessions = Arc::downgrade(&self.session_pool.sessions);
        let session_waiting_channel = Arc::clone(&self.session_waiting_channel);
        let idle_timeout = self.config.idle_timeout;
        let session_alive_trust_duration = self.config.session_alive_trust_duration;

        tokio::spawn(async move {
            loop {
                let _ = interval.tick().await;

                let max_removing_count = num_opened.load(Ordering::SeqCst) - max_idle as i64;
                if max_removing_count < 0 {
                    continue;
                }

                let now = Instant::now();
                let removed_count = match sessions.upgrade() {
                    Some(g) => {
                        // First shrink needless idle sessions.
                        let removed_count = shrink_idle_sessions(
                            now,
                            idle_timeout,
                            Arc::clone(&g),
                            max_removing_count,
                        )
                        .await;
                        // Ping request for alive sessions.
                        removed_count
                            + health_check(
                                now + std::time::Duration::from_nanos(1),
                                session_alive_trust_duration,
                                g,
                                &session_waiting_channel,
                            )
                            .await
                    }
                    None => {
                        log::error!("sessions already released");
                        0
                    }
                };

                if removed_count > 0 {
                    log::info!("{} idle sessions removed.", removed_count);
                    num_opened.fetch_add(-removed_count, Ordering::SeqCst);
                }
            }
        });
    }
}

async fn health_check(
    now: Instant,
    session_alive_trust_duration: Duration,
    sessions: Arc<Mutex<VecDeque<SessionHandle>>>,
    session_waiting_channel: &tokio::sync::broadcast::Sender<bool>,
) -> i64 {
    let mut removed_count = 0;
    let sleep_duration = Duration::from_millis(10);
    loop {
        sleep(sleep_duration).await;

        let mut s = {
            let mut locked = sessions.lock();
            match locked.pop_front() {
                Some(mut s) => {
                    // all the session check complete.
                    if s.last_checked_at == now {
                        locked.push_back(s);
                        break;
                    }
                    if std::cmp::max(s.last_used_at, s.last_pong_at) + session_alive_trust_duration
                        >= now
                    {
                        s.last_checked_at = now;
                        locked.push_back(s);
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
                sessions.lock().push_back(s);
                session_waiting_channel.send(true);
            }
            Err(err) => {
                log::error!("ping session err {:?}", err);
                removed_count += 1;
                delete_session(s).await;
            }
        }
    }
    return removed_count;
}

async fn shrink_idle_sessions(
    now: Instant,
    idle_timeout: Duration,
    sessions: Arc<Mutex<VecDeque<SessionHandle>>>,
    max_shrink_count: i64,
) -> i64 {
    let mut removed_count = 0;
    let sleep_duration = Duration::from_millis(10);
    loop {
        if removed_count >= max_shrink_count {
            break;
        }

        sleep(sleep_duration).await;

        // get old session
        let mut s = {
            let mut locked = sessions.lock();
            match locked.pop_front() {
                Some(mut s) => {
                    // all the session check complete.
                    if s.last_checked_at == now {
                        locked.push_back(s);
                        break;
                    }
                    if s.last_used_at + idle_timeout >= now {
                        s.last_checked_at = now;
                        locked.push_back(s);
                        continue;
                    }
                    s
                }
                None => break,
            }
        };

        removed_count += 1;
        delete_session(s).await;
    }
    return removed_count;
}

async fn delete_session(mut session: SessionHandle) {
    let session_name = session.session.name;
    log::info!("delete session {}", session_name);
    let request = DeleteSessionRequest {
        name: session_name.clone(),
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
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;
    use std::time::Instant;

    pub const DATABASE: &str =
        "projects/local-project/instances/test-instance/databases/local-database";

    #[tokio::test]
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
        sleep(Duration::from_secs(1));
        let removed = shrink_idle_sessions(
            Instant::now(),
            idle_timeout,
            Arc::clone(&sm.session_pool.sessions),
            5,
        )
        .await;

        // not expired
        assert_eq!(removed, 0);
    }

    #[tokio::test]
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
        sleep(Duration::from_secs(1));
        let removed = shrink_idle_sessions(
            Instant::now(),
            idle_timeout,
            Arc::clone(&sm.session_pool.sessions),
            100,
        )
        .await;

        // expired
        assert_eq!(removed, 5);
    }

    #[tokio::test]
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
        sleep(Duration::from_secs(1));

        let ch = Arc::clone(&sm.session_waiting_channel);
        tokio::spawn(async move {
            let result = ch.subscribe().recv().await.unwrap();
            assert_eq!(result, true);
        });
        let removed = health_check(
            Instant::now(),
            session_alive_trust_duration,
            Arc::clone(&sm.session_pool.sessions),
            &sm.session_waiting_channel,
        )
        .await;

        assert_eq!(removed, 0);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    #[tokio::test]
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
        sleep(Duration::from_secs(1));

        let ch = Arc::clone(&sm.session_waiting_channel);
        tokio::spawn(async move {
            let result = ch.subscribe().recv().await.unwrap();
            panic!("expected no session checked");
        });
        let removed = health_check(
            Instant::now(),
            session_alive_trust_duration,
            Arc::clone(&sm.session_pool.sessions),
            &sm.session_waiting_channel,
        )
        .await;

        assert_eq!(removed, 0);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    #[tokio::test]
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
                let locked = sm.session_pool.sessions.lock();
                assert_eq!(locked.len(), 0, "all the session are using");
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // idle session removed after cleanup
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        {
            let locked = sm.session_pool.sessions.lock();
            assert!(
                locked.len() == 19 || locked.len() == 20,
                "available sessions are 19 or 20 (when 19 cleaner pop session"
            );
        }
        assert_eq!(sm.idle_sessions(), 20, "num sessions are 20");
    }
}

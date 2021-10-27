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
use tokio::time::{timeout, sleep, Duration};
use tokio::sync::oneshot::{Sender,channel};

type Waiters = Mutex<VecDeque<Sender<SessionHandle>>>;
type Sessions = Mutex<VecDeque<SessionHandle>>;

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
            match {self.session_pool.waiters.lock().pop_front()} {
                Some(c) => c.send(session),
                None => Ok(self.session_pool.sessions.lock().push_back(session))
            };
        } else {
            let prev = self.session_pool.num_opened.fetch_add(-1, Ordering::SeqCst);
            if prev == 1 && !self.session_pool.waiters.lock().is_empty() {
                println!("should request batch creation");
            }
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
    sessions: Arc<Sessions>,
    num_opened: Arc<AtomicI64>,
    waiters: Arc<Waiters>,
}

impl SessionPool {
    fn new(
        init_pool: VecDeque<SessionHandle>,
        waiters: Arc<Waiters>,
    ) -> Self {
        let size = init_pool.len() as i64;
        SessionPool {
            sessions: Arc::new(Mutex::new(init_pool)),
            num_opened: Arc::new(AtomicI64::new(size)),
            waiters,
        }
    }
}

impl Clone for SessionPool {
    fn clone(&self) -> Self {
        SessionPool {
            sessions: Arc::clone(&self.sessions),
            num_opened: Arc::clone(&self.num_opened),
            waiters: Arc::clone(&self.waiters),
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
    waiters: Arc<Waiters>,
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

        let waiters =  Arc::new(Waiters::new(VecDeque::new()));
        return Ok(SessionManager {
            database: database_name,
            config,
            conn_pool,
            session_pool: SessionPool::new(init_pool, Arc::clone(&waiters)),
            waiters,
        });
    }

    pub fn idle_sessions(&self) -> i64 {
        self.session_pool.num_opened.load(Ordering::SeqCst)
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
        loop {
            {
                let mut locked = self.session_pool.sessions.lock();
                if let Some(mut s) = locked.pop_front() {
                    s.last_used_at = Instant::now();
                    return Ok(ManagedSession::new(self.session_pool.clone(), s));
                }
            };

            // TODO ブロードキャストはやめてoneshot channelをMutex<VecDequeue>に入れるようにする。
            // TODO ManagedSession#Dropとbatch_createの両方でdequeueしてsendする。batch生成で不足分はNoneをsendしない。
            //  -> MaxOpenまで残りがあり残waitersがいたらさらにバッチ追加読み込み。MaxOpen余力がなくなければ何もしない。Noneで通知もしない。↓のタイムアウトに引っかかる。
            // TODO timeoutはこれ。https://teaclave.apache.org/api-docs/crates-app/tokio/time/fn.timeout.html

            let (sender, receiver) = channel();
            let is_empty = {
                let mut waiters = self.waiters.lock();
                let is_empty = waiters.is_empty();
                waiters.push_back(sender);
                is_empty
            };

            if is_empty {
                self.schedule_batch_create();
            }

            // Wait for the session creation.
            return match timeout(self.config.session_get_timeout, receiver).await {
                Ok(Ok(result)) => {
                    println!("notified {}", self.idle_sessions());
                    Ok(ManagedSession {
                        session_pool: self.session_pool.clone(),
                        session: Some(result),
                    })
                }
                _ => Err(SessionError::SessionGetTimeout),
            }
        }

    }

    fn schedule_batch_create(&self) {
        let mut creation_count =
            self.config.max_opened - self.session_pool.num_opened.load(Ordering::SeqCst) as usize;
        if creation_count > self.config.inc_step {
            creation_count = self.config.inc_step;
        }

        let database = self.database.clone();
        let sessions = Arc::downgrade(&self.session_pool.sessions);
        let next_client = self.conn_pool.conn();
        let waiters = Arc::downgrade(&self.waiters);
        let num_opened = Arc::clone(&self.session_pool.num_opened);
        if creation_count == 0 {
            log::warn!("maximum session opened");
            return;
        }

        tokio::spawn(async move {
            println!("start batch create session {}", creation_count);

            let waiters = match waiters.upgrade() {
                Some(w) =>w,
                None => return
            };
            let sessions = match sessions.upgrade() {
                Some(w) =>w,
                None => return
            };

            match batch_create_session(next_client, database, creation_count).await {
                Ok(mut fresh_sessions) => {
                    while let Some(session) = fresh_sessions.pop() {
                        match { waiters.lock().pop_front() } {
                            Some(w) => w.send(session),
                            None => Ok(sessions.lock().push_back(session))
                        };
                    }
                    //Update idle session count
                    num_opened.fetch_add(creation_count as i64, Ordering::SeqCst);
                }
                Err(e) => log::error!("failed to batch creation request {:?}", e)
            };
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
        let waiters = Arc::downgrade(&self.waiters);
        let idle_timeout = self.config.idle_timeout;
        let session_alive_trust_duration = self.config.session_alive_trust_duration;

        tokio::spawn(async move {
            loop {
                let _ = interval.tick().await;

                let max_removing_count = num_opened.load(Ordering::SeqCst) - max_idle as i64;
                if max_removing_count < 0 {
                    continue;
                }

                let waiters = match waiters.upgrade() {
                    Some(w) =>w,
                    None => return
                };
                let sessions = match sessions.upgrade() {
                    Some(w) =>w,
                    None => return
                };

                let now = Instant::now();
                let mut removed_count = shrink_idle_sessions(now, idle_timeout, Arc::clone(&sessions), max_removing_count).await;
                removed_count += health_check(now + std::time::Duration::from_nanos(1), session_alive_trust_duration, sessions, waiters).await;

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
    sessions: Arc<Sessions>,
    waiters: Arc<Waiters>,
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
                match {waiters.lock().pop_front()} {
                    Some(c) => c.send(s),
                    None => Ok(sessions.lock().push_back(s))
                };
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
    sessions: Arc<Sessions>,
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
    use std::time::Instant;
    use tokio::time::{sleep, Duration};
    use std::sync::atomic::{AtomicI64, Ordering};

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

        let removed = health_check(
            Instant::now(),
            session_alive_trust_duration,
            Arc::clone(&sm.session_pool.sessions),
            Arc::clone(&sm.waiters),
        )
        .await;

        assert_eq!(removed, 0);
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

        let removed = health_check(
            Instant::now(),
            session_alive_trust_duration,
            Arc::clone(&sm.session_pool.sessions),
            Arc::clone(&sm.waiters),
        )
        .await;

        assert_eq!(removed, 0);
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
                let locked = sm.session_pool.sessions.lock();
                assert_eq!(locked.len(), 0, "all the session are using");
            }
            sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // idle session removed after cleanup
        sleep(tokio::time::Duration::from_secs(3)).await;
        {
            let locked = sm.session_pool.sessions.lock();
            assert!(
                locked.len() == 19 || locked.len() == 20,
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
        let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());

        let counter = Arc::new(AtomicI64::new(0));
        for _ in 0..100 {
            let sm = sm.clone();
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                let mut session = sm.get().await.unwrap();
        //        session.invalidate().await;
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }
        while counter.load(Ordering::SeqCst) < 100 {
            sleep(Duration::from_millis(5)).await;
        }

        assert_eq!(sm.idle_sessions(), 0);
        let locked = sm.session_pool.sessions.lock();
        assert_eq!(locked.len(), 0, "must be no session");
    }
}

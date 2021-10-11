use parking_lot::Mutex;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Weak};

use crate::apiv1::conn_pool::{ConnPool, ConnectionManager};
use crate::apiv1::spanner_client::{ping_query_request, Client};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime};
use internal::spanner::v1::spanner_client::SpannerClient;
use internal::spanner::v1::{
    BatchCreateSessionsRequest, CreateSessionRequest, DeleteSessionRequest, ExecuteSqlRequest,
    Session,
};
use oneshot::{RecvError, Sender};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::time::Instant;
use thiserror;
use tokio::sync::MutexGuard;
use tokio::task::JoinHandle;
use tokio::time::error::Elapsed;
use tokio::time::Interval;
use tonic::metadata::KeyAndValueRef;
use tonic::transport::Channel;
use tonic::Code;
use tonic::Status;

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
            Ok(s) => self.valid = false,
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
}

impl Clone for SessionPool {
    fn clone(&self) -> Self {
        SessionPool {
            sessions: Arc::clone(&self.sessions),
            num_opened: Arc::clone(&self.num_opened),
        }
    }
}

pub struct SessionConfig {
    pub max_opened: usize,
    pub min_opened: usize,
    pub max_idle: usize,
    pub idle_timeout: std::time::Duration,
    pub session_get_timeout: std::time::Duration,
    pub refresh_interval: std::time::Duration,
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
            session_get_timeout: std::time::Duration::from_secs(1),
            refresh_interval: std::time::Duration::from_secs(5 * 60),
        }
    }
}

pub struct SessionManager {
    database: String,
    conn_pool: ConnectionManager,
    waiters: Arc<Mutex<VecDeque<Sender<bool>>>>,
    session_pool: SessionPool,
    config: SessionConfig,
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
        let pool_size = init_pool.len() as i64;
        return Ok(SessionManager {
            database: database_name,
            config,
            conn_pool,
            waiters: Arc::new(Mutex::new(VecDeque::<Sender<bool>>::new())),
            session_pool: SessionPool {
                sessions: Arc::new(Mutex::new(init_pool)),
                num_opened: Arc::new(AtomicI64::new(pool_size)),
            },
        });
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
            match batch_create_session(
                Client::new(next_client),
                database.clone(),
                creation_count_per_channel,
            )
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
        log::info!("initial session created count = {}", sessions.len());
        Ok(sessions.into())
    }

    pub(crate) async fn get(&self) -> Result<ManagedSession, SessionError> {
        loop {
            {
                let mut locked = self.session_pool.sessions.lock();
                while let Some(mut s) = locked.pop_front() {
                    s.last_used_at = Instant::now();
                    log::info!("found session {}", s.session.name);
                    //Found valid session
                    return Ok(ManagedSession::new(self.session_pool.clone(), s));
                }
            };

            // Create session creation waiter.
            let (sender, receiver) = oneshot::channel();
            let is_empty = {
                let mut waiters = self.waiters.lock();
                let is_empty = waiters.is_empty();
                waiters.push_back(sender);
                is_empty
            };

            // Start to create session if not scheduled.
            if is_empty {
                self.schedule_batch_create();
            }

            // Wait for the session creation.
            match tokio::time::timeout(self.config.session_get_timeout, receiver).await {
                Ok(Ok(result)) => {
                    log::info!("session creation result received {}", result);
                    if !result {
                        return Err(SessionError::FailedToCreateSession);
                    }
                }
                _ => return Err(SessionError::SessionGetTimeout),
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
        let idle_sessions = Arc::downgrade(&self.session_pool.sessions);
        let waiters = Arc::downgrade(&self.waiters);
        let next_client = self.conn_pool.conn();
        let num_opened = Arc::downgrade(&self.session_pool.num_opened);

        tokio::spawn(async move {
            log::info!("start batch create session {}", creation_count);
            let result = match batch_create_session(
                Client::new(next_client),
                database,
                creation_count,
            )
            .await
            {
                Ok(mut fresh_sessions) => {
                    // Register fresh sessions into pool.
                    let result = match idle_sessions.upgrade() {
                        Some(g) => {
                            let mut locked_idle_session = g.lock();
                            while let Some(session) = fresh_sessions.pop() {
                                locked_idle_session.push_back(session);
                            }
                            true
                        }
                        None => {
                            log::error!("idle session pool already released.");
                            false
                        }
                    };
                    // Update idle session count
                    if result {
                        match num_opened.upgrade() {
                            Some(g) => {
                                g.fetch_add(creation_count as i64, Ordering::SeqCst);
                                log::info!(
                                    "current idle session count = {}",
                                    g.load(Ordering::SeqCst)
                                );
                            }
                            None => {
                                log::error!("num_opened already released.");
                            }
                        }
                    }
                    result
                }
                Err(e) => {
                    log::error!("failed to batch creation request {:?}", e);
                    false
                }
            };

            // Notify waiters blocking on session creation.
            notify_to_waiters(result, waiters);
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
        let num_opened = Arc::downgrade(&self.session_pool.num_opened);
        let sessions = Arc::downgrade(&self.session_pool.sessions);

        tokio::spawn(async move {
            loop {
                let _ = interval.tick().await;

                let max_removing_count = match num_opened.upgrade() {
                    Some(num) => num.load(Ordering::SeqCst) - max_idle as i64,
                    None => {
                        log::error!("no longer exists num_opened");
                        break;
                    }
                };
                log::info!(
                    "refresh session pool: max_removing_count={}",
                    max_removing_count
                );
                if max_removing_count < 0 {
                    continue;
                }

                let now = Instant::now();
                let removed_count = match sessions.upgrade() {
                    Some(g) => {
                        // First shrink needless idle sessions.
                        let mut removed_count =
                            shrink_idle_sessions(now, Arc::clone(&g), max_removing_count).await;
                        // Ping request for alive sessions.
                        removed_count
                            + health_check(now + std::time::Duration::from_nanos(1), g).await
                    }
                    None => {
                        log::error!("sessions already released");
                        0
                    }
                };

                if removed_count > 0 {
                    log::info!("{} idle sessions removed.", removed_count);
                    match num_opened.upgrade() {
                        Some(n) => {
                            let prev = n.fetch_add(-removed_count, Ordering::SeqCst);
                            let remains = prev - removed_count;
                            log::info!("{} current idle session remains.", remains);
                            if remains <= 0 {}
                        }
                        None => {
                            log::error!("failed to update num_opened count");
                        }
                    }
                }
            }
        });
    }
}

async fn health_check(now: Instant, sessions: Arc<Mutex<VecDeque<SessionHandle>>>) -> i64 {
    let mut removed_count = 0;
    loop {
        let mut s = {
            match sessions.lock().pop_front() {
                Some(s) => s,
                None => break,
            }
        };
        // Break if the all session checked
        if s.last_checked_at == now {
            sessions.lock().push_back(s);
            break;
        }

        //後に使ったかpingしてから指定時刻経過
        log::info!(
            "session last pong = {:?}",
            now - std::cmp::max(s.last_used_at, s.last_pong_at)
                + std::time::Duration::from_secs(60 * 15)
        );
        let mut should_ping = std::cmp::max(s.last_used_at, s.last_pong_at)
            + std::time::Duration::from_secs(60 * 15)
            < now;

        if should_ping {
            let session_name = s.session.name.clone();
            log::info!("ping session {}", session_name);
            let request = ping_query_request(session_name.clone());
            match s.spanner_client.execute_sql(request, None).await {
                Ok(_) => {
                    s.last_checked_at = now.clone();
                    s.last_pong_at = now;
                    sessions.lock().push_back(s);
                }
                Err(err) => {
                    log::error!("ping session err {}", err);
                    log::error!("ping session err message = {}", err.message());
                    log::error!("ping session err code = {}", err.code());
                    err.metadata().iter().for_each(|x| match x {
                        KeyAndValueRef::Ascii(k, v) => {
                            log::error!("ping session err metadata ascii key = {}", k.to_string())
                        }
                        KeyAndValueRef::Binary(k, v) => {
                            log::error!("ping session err metadata binary key= {}", k.to_string())
                        }
                    });
                    removed_count += 1;
                    log::info!("delete session {}", session_name);
                    let request = DeleteSessionRequest { name: session_name };
                    s.spanner_client.delete_session(request, None).await;
                }
            }
        } else {
            s.last_checked_at = now;
            sessions.lock().push_back(s);
        }
    }
    return removed_count;
}

async fn shrink_idle_sessions(
    now: Instant,
    sessions: Arc<Mutex<VecDeque<SessionHandle>>>,
    max_shrink_count: i64,
) -> i64 {
    let mut removed_count = 0;
    loop {
        //Break if the sufficient idle session removed.
        if removed_count >= max_shrink_count {
            break;
        }

        let mut s = {
            match sessions.lock().pop_front() {
                Some(s) => s,
                None => break,
            }
        };
        // Break if the all session checked
        if s.last_checked_at == now {
            sessions.lock().push_back(s);
            break;
        }

        //生成 or 最後に使ってから指定時刻経過
        log::info!(
            "shrink target session last_used_at = {:?}",
            now - s.last_used_at + std::time::Duration::from_secs(60 * 30)
        );
        let mut should_remove = s.last_used_at + std::time::Duration::from_secs(60 * 30) < now;

        if should_remove {
            removed_count += 1;
            delete_session(s).await;
        } else {
            s.last_checked_at = now;
            sessions.lock().push_back(s);
        }
    }
    return removed_count;
}

async fn delete_session(mut session: SessionHandle) {
    log::info!("delete session {}", session.session.name);
    let request = DeleteSessionRequest {
        name: session.session.name.clone(),
    };
    session.spanner_client.delete_session(request, None).await;
}

fn notify_to_waiters(result: bool, waiters: Weak<Mutex<VecDeque<Sender<bool>>>>) {
    match waiters.upgrade() {
        Some(g) => {
            let mut locked_waiters = g.lock();
            while let Some(waiter) = locked_waiters.pop_front() {
                waiter.send(result);
            }
        }
        None => log::error!("waiters already released."),
    };
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
        .map(|s| SessionHandle {
            last_checked_at: now,
            last_used_at: now,
            last_pong_at: now,
            session: s,
            spanner_client: spanner_client.clone(),
            valid: true,
        })
        .collect::<Vec<SessionHandle>>());
}

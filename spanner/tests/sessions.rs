







use serial_test::serial;


mod common;
use common::*;
use google_cloud_spanner::apiv1::conn_pool::ConnectionManager;

use google_cloud_spanner::sessions::{SessionConfig, SessionError, SessionManager};


#[tokio::test]
#[serial]
async fn test_init_pool() {
    let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
        .await
        .unwrap();
    let mut config = SessionConfig::default();
    config.min_opened = 1;
    config.max_opened = 26;
    let sm = SessionManager::new(DATABASE, cm, config).await.unwrap();
    let idle_sessions = sm.num_opened();
    assert_eq!(idle_sessions, 1);
}

#[tokio::test]
#[serial]
async fn test_grow_session() {
    let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
        .await
        .unwrap();
    let mut config = SessionConfig::default();
    config.min_opened = 1;
    config.max_opened = 26;
    let sm = SessionManager::new(DATABASE, cm, config).await.unwrap();
    let mut sessions = Vec::with_capacity(26);
    for _ in 0..26 {
        sessions.push(sm.get().await.unwrap());
    }
    let idle_sessions = sm.num_opened();
    assert_eq!(idle_sessions, 26);
}

#[tokio::test]
#[serial]
async fn test_grow_timeout() {
    let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
        .await
        .unwrap();
    let mut config = SessionConfig::default();
    config.min_opened = 1;
    config.max_opened = 2;
    let sm = SessionManager::new(DATABASE, cm, config).await.unwrap();
    let _s1 = sm.get().await.unwrap();
    let _s2 = sm.get().await.unwrap();
    match sm.get().await {
        Ok(_s) => panic!("must be error"),
        Err(e) => match e {
            SessionError::SessionGetTimeout => {}
            oth => panic!("invalid error {:?}", oth),
        },
    };
    let idle_sessions = sm.num_opened();
    assert_eq!(idle_sessions, 2);
}

#[tokio::test]
#[serial]
async fn test_grow_wait_and_get() {
    let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
        .await
        .unwrap();
    let mut config = SessionConfig::default();
    config.min_opened = 1;
    config.max_opened = 2;
    let sm = std::sync::Arc::new(SessionManager::new(DATABASE, cm, config).await.unwrap());
    {
        let cloned = sm.clone();
        let _s1 = cloned.get().await.unwrap();
        let _s2 = cloned.get().await.unwrap();
        tokio::spawn(async move {
            match cloned.clone().get().await {
                Ok(_s) => {
                    println!("session available");
                }
                Err(e) => panic!("invalid error {:?}", e),
            };
        });
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    assert_eq!(sm.num_opened(), 2);
    assert_eq!(sm.session_waiters(), 0);
}

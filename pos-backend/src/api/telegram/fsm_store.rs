use crate::db;
use serde::{Deserialize, Serialize};

pub const FSM_TTL_SECS: u64 = 900; // 15 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSessionPayload {
    pub item_name: String,
    pub currency: Option<String>,
}

/// Async SQLite-backed FSM store utilizing deadpool-sqlite pool.
#[derive(Clone)]
pub struct FsmStore {
    db_path: String,
    pool: Option<deadpool_sqlite::Pool>,
}

impl Default for FsmStore {
    fn default() -> Self {
        let db_path = "data/pos_store.db".to_string();
        let pool = db::create_db_pool(&db_path).ok();
        Self { db_path, pool }
    }
}

impl FsmStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_db(db_path: String) -> Self {
        let pool = db::create_db_pool(&db_path).ok();
        Self { db_path, pool }
    }

    pub fn new_with_pool(db_path: String, pool: deadpool_sqlite::Pool) -> Self {
        Self {
            db_path,
            pool: Some(pool),
        }
    }

    pub fn pool(&self) -> Option<&deadpool_sqlite::Pool> {
        self.pool.as_ref()
    }

    /// Sets or updates pending session state for (chat_id, user_id).
    pub async fn set_pending(
        &self,
        chat_id: i64,
        user_id: i64,
        item_name: String,
        currency: Option<String>,
    ) {
        let payload = PendingSessionPayload {
            item_name,
            currency,
        };
        let json_str = serde_json::to_string(&payload).unwrap_or_default();

        if let Some(ref pool) = self.pool {
            if let Ok(conn) = pool.get().await {
                let _ = conn
                    .interact(move |c| {
                        let _ = db::fsm_dao::set_session(
                            c,
                            chat_id,
                            user_id,
                            "AWAITING_PRICE",
                            &json_str,
                        );
                    })
                    .await;
                return;
            }
        }

        let db_path = self.db_path.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = db::get_db_connection(&db_path) {
                let _ =
                    db::fsm_dao::set_session(&conn, chat_id, user_id, "AWAITING_PRICE", &json_str);
            }
        })
        .await;
    }

    /// Gets valid pending session payload for (chat_id, user_id) if within TTL.
    pub async fn get_pending(&self, chat_id: i64, user_id: i64) -> Option<PendingSessionPayload> {
        if let Some(ref pool) = self.pool {
            if let Ok(conn) = pool.get().await {
                let res = conn
                    .interact(move |c| {
                        if let Ok(Some((_state, payload_json))) =
                            db::fsm_dao::get_session(c, chat_id, user_id, FSM_TTL_SECS)
                        {
                            return serde_json::from_str::<PendingSessionPayload>(&payload_json)
                                .ok();
                        }
                        None
                    })
                    .await;
                return res.unwrap_or(None);
            }
        }

        let db_path = self.db_path.clone();
        let res = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = db::get_db_connection(&db_path) {
                if let Ok(Some((_state, payload_json))) =
                    db::fsm_dao::get_session(&conn, chat_id, user_id, FSM_TTL_SECS)
                {
                    return serde_json::from_str::<PendingSessionPayload>(&payload_json).ok();
                }
            }
            None
        })
        .await;

        res.unwrap_or(None)
    }

    /// Removes session state for (chat_id, user_id).
    pub async fn clear(&self, chat_id: i64, user_id: i64) {
        if let Some(ref pool) = self.pool {
            if let Ok(conn) = pool.get().await {
                let _ = conn
                    .interact(move |c| {
                        let _ = db::fsm_dao::clear_session(c, chat_id, user_id);
                    })
                    .await;
                return;
            }
        }

        let db_path = self.db_path.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = db::get_db_connection(&db_path) {
                let _ = db::fsm_dao::clear_session(&conn, chat_id, user_id);
            }
        })
        .await;
    }

    /// Removes all session states for a given chat_id regardless of user_id.
    pub async fn clear_chat(&self, chat_id: i64) {
        if let Some(ref pool) = self.pool {
            if let Ok(conn) = pool.get().await {
                let _ = conn
                    .interact(move |c| {
                        let _ = db::fsm_dao::clear_all_chat_sessions(c, chat_id);
                    })
                    .await;
                return;
            }
        }

        let db_path = self.db_path.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = db::get_db_connection(&db_path) {
                let _ = db::fsm_dao::clear_all_chat_sessions(&conn, chat_id);
            }
        })
        .await;
    }
}

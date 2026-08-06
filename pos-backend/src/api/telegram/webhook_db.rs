use crate::db;
use deadpool_sqlite::Pool;

/// Enqueues an incoming webhook update payload to SQLite with deadline fallback support.
pub async fn enqueue_update_payload(
    db_pool: Option<&Pool>,
    db_path: &str,
    update_id: i64,
    chat_id: Option<i64>,
    payload_str: &str,
) -> Result<bool, String> {
    let p_str = payload_str.to_string();
    if let Some(pool) = db_pool {
        // Enforce 4500ms pool acquire timeout to return HTTP 500 fast before Telegram gateway timeout
        let conn_res =
            tokio::time::timeout(std::time::Duration::from_millis(4500), pool.get()).await;

        let conn = match conn_res {
            Ok(Ok(c)) => c,
            Ok(Err(e)) => return Err(format!("Pool acquire error: {}", e)),
            Err(_) => return Err("Pool acquire timed out (4500ms)".to_string()),
        };

        conn.interact(move |c| db::updates::enqueue_webhook_update(c, update_id, chat_id, &p_str))
            .await
            .map_err(|e| format!("Interact error: {}", e))?
            .map_err(|e| format!("DB error: {}", e))
    } else {
        let db_p = db_path.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db::get_db_connection(&db_p).map_err(|e| e.to_string())?;
            db::updates::enqueue_webhook_update(&conn, update_id, chat_id, &p_str)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| format!("Spawn error: {}", e))?
    }
}

/// Marks a processed webhook update as completed in SQLite.
pub async fn mark_done(
    db_pool: Option<&Pool>,
    db_path: &str,
    update_id: i64,
) -> Result<(), String> {
    if let Some(pool) = db_pool {
        if let Ok(conn) = pool.get().await {
            let _ = conn
                .interact(move |c| db::updates::mark_webhook_update_done(c, update_id))
                .await;
            return Ok(());
        }
    }

    let db_p = db_path.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        if let Ok(conn) = db::get_db_connection(&db_p) {
            let _ = db::updates::mark_webhook_update_done(&conn, update_id);
        }
    })
    .await;
    Ok(())
}

/// Checks if an update_id has already been processed in SQLite.
pub async fn is_update_processed(db_pool: Option<&Pool>, db_path: &str, update_id: i64) -> bool {
    if let Some(pool) = db_pool {
        if let Ok(conn) = pool.get().await {
            let res = conn
                .interact(move |c| db::updates::is_processed(c, update_id).unwrap_or(false))
                .await;
            return res.unwrap_or(false);
        }
    }

    let db_p = db_path.to_string();
    tokio::task::spawn_blocking(move || {
        if let Ok(conn) = db::get_db_connection(&db_p) {
            db::updates::is_processed(&conn, update_id).unwrap_or(false)
        } else {
            false
        }
    })
    .await
    .unwrap_or(false)
}

/// Records a webhook update failure, scheduling exponential backoff retry or DLQ movement.
pub async fn record_failure(
    db_pool: Option<&Pool>,
    db_path: &str,
    update_id: i64,
    chat_id: Option<i64>,
    payload_str: &str,
    err_msg: &str,
    max_retries: i32,
) -> bool {
    let p_str = payload_str.to_string();
    let e_str = err_msg.to_string();

    if let Some(pool) = db_pool {
        if let Ok(conn) = pool.get().await {
            return conn
                .interact(move |c| {
                    db::updates::record_webhook_failure(
                        c,
                        update_id,
                        chat_id,
                        &p_str,
                        &e_str,
                        max_retries,
                    )
                    .unwrap_or(false)
                })
                .await
                .unwrap_or(false);
        }
    }

    let db_p = db_path.to_string();
    tokio::task::spawn_blocking(move || {
        if let Ok(conn) = db::get_db_connection(&db_p) {
            db::updates::record_webhook_failure(
                &conn,
                update_id,
                chat_id,
                &p_str,
                &e_str,
                max_retries,
            )
            .unwrap_or(false)
        } else {
            false
        }
    })
    .await
    .unwrap_or(false)
}

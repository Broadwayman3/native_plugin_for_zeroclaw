use axum::extract::State;
use axum::Json;

use crate::db;
use crate::error::AppError;

/// GET /api/v1/sales/summary - Aggregated sales metrics
pub async fn handle_sales_summary(
    State(state): State<crate::api::AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;
    let summary = db::invoices::get_sales_summary(&conn)?;
    Ok(Json(summary))
}

use std::sync::Arc;
use axum::{ extract::{ Path, Query, State }, http::StatusCode, response::IntoResponse, Json };
use serde::Deserialize;
use uuid::Uuid;

use super::use_cases::AdminUseCase;

pub type AdminState = Arc<AdminUseCase>;

#[derive(Deserialize)]
pub struct LimitQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

// ─── Merchants ────────────────────────────────────────────────────────────────

pub async fn list_merchants(State(state): State<AdminState>) -> impl IntoResponse {
    match state.list_merchants().await {
        Ok(merchants) => Json(serde_json::json!({ "merchants": merchants })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_merchant(
    State(state): State<AdminState>,
    Path(id): Path<Uuid>
) -> impl IntoResponse {
    match state.get_merchant(id).await {
        Ok(m) => Json(m).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn activate_merchant(
    State(state): State<AdminState>,
    Path(id): Path<Uuid>
) -> impl IntoResponse {
    match state.activate_merchant(id, "admin".to_string()).await {
        Ok(m) => Json(m).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn deactivate_merchant(
    State(state): State<AdminState>,
    Path(id): Path<Uuid>
) -> impl IntoResponse {
    match state.deactivate_merchant(id, "admin".to_string()).await {
        Ok(m) => Json(m).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// ─── Invoices ─────────────────────────────────────────────────────────────────

pub async fn list_merchant_invoices(
    State(state): State<AdminState>,
    Path(merchant_id): Path<Uuid>
) -> impl IntoResponse {
    match state.list_invoices_by_merchant(merchant_id).await {
        Ok(items) => Json(serde_json::json!({ "invoices": items })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── Payments ─────────────────────────────────────────────────────────────────

pub async fn list_merchant_payments(
    State(state): State<AdminState>,
    Path(merchant_id): Path<Uuid>
) -> impl IntoResponse {
    match state.list_payments_by_merchant(merchant_id).await {
        Ok(items) => Json(serde_json::json!({ "payments": items })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── Settlements ──────────────────────────────────────────────────────────────

pub async fn list_merchant_settlements(
    State(state): State<AdminState>,
    Path(merchant_id): Path<Uuid>
) -> impl IntoResponse {
    match state.list_settlements_by_merchant(merchant_id).await {
        Ok(items) => Json(serde_json::json!({ "settlements": items })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn trigger_settlements(State(state): State<AdminState>) -> impl IntoResponse {
    match state.trigger_settlement_processing("admin".to_string()).await {
        Ok(count) => Json(serde_json::json!({ "processed": count })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── Webhooks ─────────────────────────────────────────────────────────────────

pub async fn list_pending_webhooks(State(state): State<AdminState>) -> impl IntoResponse {
    match state.list_pending_webhook_events().await {
        Ok(items) => Json(serde_json::json!({ "events": items })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn retry_webhooks(State(state): State<AdminState>) -> impl IntoResponse {
    match state.retry_webhooks("admin".to_string()).await {
        Ok(_) => Json(serde_json::json!({ "status": "ok" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── Audit Logs ───────────────────────────────────────────────────────────────

pub async fn get_audit_logs(
    State(state): State<AdminState>,
    Query(q): Query<LimitQuery>
) -> impl IntoResponse {
    let logs = state.get_recent_audit_logs(q.limit).await;
    Json(serde_json::json!({ "logs": logs })).into_response()
}

pub async fn get_merchant_audit_logs(
    State(state): State<AdminState>,
    Path(merchant_id): Path<Uuid>,
    Query(q): Query<LimitQuery>
) -> impl IntoResponse {
    let logs = state.get_merchant_audit_logs(merchant_id, q.limit).await;
    Json(serde_json::json!({ "logs": logs })).into_response()
}

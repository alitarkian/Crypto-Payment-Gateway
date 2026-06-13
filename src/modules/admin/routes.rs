use axum::{ routing::{ get, post, put }, Router };
use super::handlers::*;

pub fn admin_routes(state: AdminState) -> Router {
    Router::new()
        // Merchants
        .route("/api/v1/admin/merchants", get(list_merchants))
        .route("/api/v1/admin/merchants/{id}", get(get_merchant))
        .route("/api/v1/admin/merchants/{id}/activate", put(activate_merchant))
        .route("/api/v1/admin/merchants/{id}/deactivate", put(deactivate_merchant))
        // Invoices
        .route("/api/v1/admin/merchants/{id}/invoices", get(list_merchant_invoices))
        // Payments
        .route("/api/v1/admin/merchants/{id}/payments", get(list_merchant_payments))
        // Settlements
        .route("/api/v1/admin/merchants/{id}/settlements", get(list_merchant_settlements))
        .route("/api/v1/admin/settlements/trigger", post(trigger_settlements))
        // Webhooks
        .route("/api/v1/admin/webhooks/pending", get(list_pending_webhooks))
        .route("/api/v1/admin/webhooks/retry", post(retry_webhooks))
        // Audit logs
        .route("/api/v1/admin/audit-logs", get(get_audit_logs))
        .route("/api/v1/admin/merchants/{id}/audit-logs", get(get_merchant_audit_logs))
        .with_state(state)
}

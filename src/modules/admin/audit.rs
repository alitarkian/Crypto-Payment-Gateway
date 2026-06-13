use chrono::{ DateTime, Utc };
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub action: String,
    pub actor: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub merchant_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct LogAction {
    pub action: &'static str,
    pub actor: String,
    pub entity_type: &'static str,
    pub entity_id: Uuid,
    pub merchant_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
}

#[derive(Clone)]
pub struct AuditLogger {
    pool: PgPool,
}

impl AuditLogger {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log(&self, cmd: LogAction) {
        let result = sqlx
            ::query(
                r#"INSERT INTO audit_logs
                (action, actor, entity_type, entity_id, merchant_id, metadata, ip_address)
               VALUES ($1::text::audit_action, $2, $3, $4, $5, $6, $7)"#
            )
            .bind(cmd.action)
            .bind(&cmd.actor)
            .bind(cmd.entity_type)
            .bind(cmd.entity_id)
            .bind(cmd.merchant_id)
            .bind(cmd.metadata.as_ref())
            .bind(cmd.ip_address.as_deref())
            .execute(&self.pool).await;

        if let Err(e) = result {
            tracing::error!(error = %e, action = cmd.action, "Failed to write audit log");
        }
    }

    pub async fn find_by_merchant(&self, merchant_id: Uuid, limit: i64) -> Vec<AuditLog> {
        sqlx::query_as::<_, AuditLog>(
            r#"SELECT id, action::TEXT as action, actor, entity_type, entity_id,
                      merchant_id, metadata, ip_address, created_at
               FROM audit_logs
               WHERE merchant_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#
        )
            .bind(merchant_id)
            .bind(limit)
            .fetch_all(&self.pool).await
            .unwrap_or_default()
    }

    pub async fn find_recent(&self, limit: i64) -> Vec<AuditLog> {
        sqlx::query_as::<_, AuditLog>(
            r#"SELECT id, action::TEXT as action, actor, entity_type, entity_id,
                      merchant_id, metadata, ip_address, created_at
               FROM audit_logs
               ORDER BY created_at DESC
               LIMIT $1"#
        )
            .bind(limit)
            .fetch_all(&self.pool).await
            .unwrap_or_default()
    }
}

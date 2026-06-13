-- Add migration script here
CREATE TYPE audit_action AS ENUM (
    'merchant.created',
    'merchant.activated',
    'merchant.deactivated',
    'invoice.created',
    'invoice.paid',
    'invoice.expired',
    'payment.detected',
    'payment.processed',
    'settlement.created',
    'settlement.processed',
    'webhook.registered',
    'webhook.delivered',
    'webhook.failed',
    'admin.merchant_activated',
    'admin.merchant_deactivated',
    'admin.settlement_triggered',
    'admin.webhook_retried'
);

CREATE TABLE audit_logs (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action       audit_action NOT NULL,
    actor        VARCHAR(100) NOT NULL DEFAULT 'system',
    entity_type  VARCHAR(100) NOT NULL,
    entity_id    UUID NOT NULL,
    merchant_id  UUID REFERENCES merchants(id) ON DELETE SET NULL,
    metadata     JSONB,
    ip_address   VARCHAR(45),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_merchant_id ON audit_logs(merchant_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
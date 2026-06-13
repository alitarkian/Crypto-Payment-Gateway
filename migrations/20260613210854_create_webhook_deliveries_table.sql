-- Add migration script here
CREATE TABLE webhook_deliveries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_event_id UUID NOT NULL REFERENCES webhook_events(id) ON DELETE CASCADE,
    webhook_id      UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    attempt         INT NOT NULL DEFAULT 1,
    status_code     INT,
    response_body   TEXT,
    error_message   TEXT,
    delivered_at    TIMESTAMPTZ,
    next_retry_at   TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhook_deliveries_event_id ON webhook_deliveries(webhook_event_id);
CREATE INDEX idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at) 
    WHERE delivered_at IS NULL;
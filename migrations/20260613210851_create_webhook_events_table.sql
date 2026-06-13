-- Add migration script here
CREATE TYPE webhook_event_type AS ENUM (
    'invoice.paid',
    'invoice.expired',
    'payment.detected'
);

CREATE TYPE webhook_event_status AS ENUM (
    'pending',
    'delivered',
    'failed'
);

CREATE TABLE webhook_events (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id  UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    event_type   webhook_event_type NOT NULL,
    payload      JSONB NOT NULL,
    status       webhook_event_status NOT NULL DEFAULT 'pending',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhook_events_merchant_id ON webhook_events(merchant_id);
CREATE INDEX idx_webhook_events_status ON webhook_events(status);
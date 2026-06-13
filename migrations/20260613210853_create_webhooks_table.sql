-- Add migration script here
CREATE TABLE webhooks (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id  UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    url          VARCHAR(2048) NOT NULL,
    secret       VARCHAR(255) NOT NULL,
    is_active    BOOLEAN NOT NULL DEFAULT true,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhooks_merchant_id ON webhooks(merchant_id);
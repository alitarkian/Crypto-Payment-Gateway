-- Add migration script here
CREATE TYPE batch_status AS ENUM (
    'open',
    'closed',
    'processing',
    'completed',
    'failed'
);

CREATE TABLE settlement_batches (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id     UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    status          batch_status NOT NULL DEFAULT 'open',
    total_gross     NUMERIC(20, 8) NOT NULL DEFAULT 0,
    total_fee       NUMERIC(20, 8) NOT NULL DEFAULT 0,
    total_net       NUMERIC(20, 8) NOT NULL DEFAULT 0,
    settlement_count INT NOT NULL DEFAULT 0,
    asset           VARCHAR(50) NOT NULL DEFAULT 'USDC',
    blockchain      VARCHAR(50) NOT NULL DEFAULT 'solana',
    period_date     DATE NOT NULL,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_batches_merchant_period 
    ON settlement_batches(merchant_id, period_date) 
    WHERE status = 'open';
CREATE INDEX idx_batches_status ON settlement_batches(status);
CREATE INDEX idx_batches_merchant_id ON settlement_batches(merchant_id);
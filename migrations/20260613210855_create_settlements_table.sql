-- Add migration script here
CREATE TYPE settlement_status AS ENUM (
    'pending',
    'processing',
    'ready_to_pay',
    'paid',
    'failed'
);

CREATE TABLE settlements (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id     UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    invoice_id      UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    payment_id      UUID NOT NULL REFERENCES payments(id) ON DELETE CASCADE,
    gross_amount    NUMERIC(20, 8) NOT NULL,
    fee_rate        NUMERIC(5, 4) NOT NULL DEFAULT 0.0100,
    fee_amount      NUMERIC(20, 8) NOT NULL,
    net_amount      NUMERIC(20, 8) NOT NULL,
    asset           VARCHAR(50) NOT NULL DEFAULT 'USDC',
    blockchain      VARCHAR(50) NOT NULL DEFAULT 'solana',
    status          settlement_status NOT NULL DEFAULT 'pending',
    batch_id        UUID,
    settled_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_settlements_payment_id ON settlements(payment_id);
CREATE INDEX idx_settlements_merchant_id ON settlements(merchant_id);
CREATE INDEX idx_settlements_status ON settlements(status);
CREATE INDEX idx_settlements_batch_id ON settlements(batch_id);
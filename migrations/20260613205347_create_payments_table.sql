-- Add migration script here
CREATE TYPE payment_status AS ENUM (
    'detected',
    'confirmed',
    'failed'
);

CREATE TABLE payments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id      UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    wallet_id       UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    merchant_id     UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    signature       VARCHAR(255) NOT NULL UNIQUE,
    amount          NUMERIC(20, 8) NOT NULL,
    asset           VARCHAR(50) NOT NULL DEFAULT 'USDC',
    blockchain      VARCHAR(50) NOT NULL DEFAULT 'solana',
    status          payment_status NOT NULL DEFAULT 'detected',
    detected_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_invoice_id ON payments(invoice_id);
CREATE INDEX idx_payments_merchant_id ON payments(merchant_id);
CREATE INDEX idx_payments_signature ON payments(signature);
CREATE INDEX idx_payments_status ON payments(status);
-- Add migration script here
CREATE TYPE invoice_status AS ENUM (
    'draft',
    'pending',
    'paid',
    'expired',
    'cancelled'
);

CREATE TABLE invoices (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id     UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    wallet_id       UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    amount          NUMERIC(20, 8) NOT NULL,
    asset           VARCHAR(50) NOT NULL DEFAULT 'USDC',
    blockchain      VARCHAR(50) NOT NULL DEFAULT 'solana',
    status          invoice_status NOT NULL DEFAULT 'pending',
    description     TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    paid_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoices_merchant_id ON invoices(merchant_id);
CREATE INDEX idx_invoices_wallet_id ON invoices(wallet_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_expires_at ON invoices(expires_at);
-- Add migration script here
CREATE TABLE wallets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id     UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    address         VARCHAR(255) NOT NULL UNIQUE,
    blockchain      VARCHAR(50)  NOT NULL DEFAULT 'solana',
    asset           VARCHAR(50)  NOT NULL DEFAULT 'USDC',
    balance         NUMERIC(20, 8) NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wallets_merchant_id ON wallets(merchant_id);
CREATE INDEX idx_wallets_address ON wallets(address);
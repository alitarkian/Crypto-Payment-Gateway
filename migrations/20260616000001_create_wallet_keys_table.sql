-- wallet_keys: stores AES-256-GCM encrypted private keys for managed (custodial) wallets.
--
-- Security invariants:
--   1. encrypted_private_key is NEVER stored plaintext.
--   2. key_nonce is unique per row (GCM nonce reuse breaks confidentiality).
--   3. This table must NEVER be exposed via API — no SELECT in handlers.
--   4. Access only through WalletVault service.
--   5. Row-level access should be restricted at the DB user level in production.

CREATE TABLE wallet_keys (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id               UUID NOT NULL UNIQUE REFERENCES wallets(id) ON DELETE CASCADE,
    -- AES-256-GCM ciphertext (private key bytes, encrypted)
    encrypted_private_key   BYTEA NOT NULL,
    -- AES-GCM nonce — 12 bytes, unique per encryption operation
    key_nonce               BYTEA NOT NULL,
    -- Key version — used to track which master key version was used (for rotation)
    key_version             INTEGER NOT NULL DEFAULT 1,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- No additional indexes needed — wallet_id is UNIQUE (already indexed by constraint)
-- and we only ever look up by wallet_id

COMMENT ON TABLE wallet_keys IS
    'Encrypted private keys for managed wallets. Never expose via API.';
COMMENT ON COLUMN wallet_keys.encrypted_private_key IS
    'AES-256-GCM ciphertext of the raw private key bytes.';
COMMENT ON COLUMN wallet_keys.key_nonce IS
    '12-byte GCM nonce used during encryption. Unique per row.';
COMMENT ON COLUMN wallet_keys.key_version IS
    'Master key version used for encryption, enables key rotation.';

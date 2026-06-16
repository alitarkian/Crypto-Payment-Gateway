# Changelog

All notable changes to this project must be documented here.

Format:

Keep a Changelog

Semantic Versioning

---

## [Unreleased]

### Added

* Initial repository planning
* Production architecture definition
* Project memory system
* Long-term roadmap

---

## [0.15.0] - Docker Deployment - 2026-06-16

### Added

* `Dockerfile` — multi-stage Rust build (builder: rust:1.87-slim, runtime: debian:bookworm-slim)
* `SQLX_OFFLINE=true` build mode — compile without live DB using `.sqlx/` query cache
* `.dockerignore` — excludes `target/`, `.git/`, local env files
* `.env.docker` — Docker-specific env vars (service hostnames instead of localhost)
* `docker-compose.yml` — updated with `gateway` app service + healthchecks + `depends_on`
* Named Docker network `crypto_gateway_net` — other projects attach to call the gateway
* Non-root `gateway` user inside container for security
* Aligned credentials: PostgreSQL and RabbitMQ now use `erpos:erpos` in Docker (matching `.env`)

---

## [0.14.1] - Compilation Fixes (Rust 2024 + Dependency Compat) - 2026-06-16

### Fixed

* **`secp256k1` feature name** — renamed `bitcoin_hashes` → `hashes` (correct feature in v0.29)
* **`aes-gcm` nonce API** — replaced deprecated `Nonce::from_slice` with `Aes256Gcm::generate_nonce(&mut OsRng)` in encrypt; `Nonce::from([u8; 12])` in decrypt
* **`zeroize` trait not in scope** — added `use zeroize::Zeroize` import and `AeadCore` to aes-gcm imports
* **`set_var` unsafe** — wrapped `std::env::set_var` calls in `unsafe {}` blocks (required in Rust 2024 edition)
* **`reqwest 0.13` `.query()` unavailable** — replaced `.query(&params)` with inline URL query string in `TronClient::get_trc20_transfers_to()`
* **Temporary borrow in array** — removed `&min_timestamp_ms.to_string()` temporary reference inside array literal

---

## [0.1.0] - Foundation - 2026-06-13

### Added

* Rust project initialization (edition 2021)
* Axum 0.8 HTTP server setup
* Health endpoint GET /health
* Configuration system (config + dotenvy)
  * AppConfig, AppSettings, DatabaseSettings, RedisSettings
  * Loaded from environment variables
* Tracing setup (tracing + tracing-subscriber)
* PostgreSQL connection pool (sqlx 0.9, PgPoolOptions)
* SQLx migrations system (./migrations)
* Infrastructure module structure
* Docker Compose with PostgreSQL, Redis, RabbitMQ

### Planned

* Merchant module
* Wallet module
* Invoice module

---

## Change Categories

Use only:

Added
Changed
Deprecated
Removed
Fixed
Security

---

## Release Rules

Patch:

Bug fixes

Minor:

Backward-compatible features

Major:

Breaking changes

---

## Deployment Rule

Every production deployment must update:

* CHANGELOG.md
* Version
* Migration records

before release.

## [0.2.0] - Merchant Module - 2026-06-13

### Added
* Merchant domain entity with validation
* MerchantRepository trait (abstraction)
* PostgresMerchantRepository (SQLx implementation)
* CreateMerchant use case with duplicate email check
* GetMerchant use case
* POST /api/v1/merchants endpoint
* GET /api/v1/merchants/{id} endpoint
* merchants table migration

## [0.3.0] - Wallet Module - 2026-06-13

### Added
* Wallet domain entity (Blockchain, Asset enums)
* WalletRepository trait
* PostgresWalletRepository (SQLx implementation)
* CreateWallet use case with duplicate address check
* GetWallet use case
* ListMerchantWallets use case
* POST /api/v1/wallets endpoint
* GET /api/v1/wallets/{id} endpoint
* GET /api/v1/merchants/{merchant_id}/wallets endpoint
* wallets table migration with merchant_id foreign key

## [0.4.0] - Invoice Module - 2026-06-13

### Added
* Invoice domain entity with status machine (Pending, Paid, Expired, Cancelled)
* InvoiceRepository trait
* PostgresInvoiceRepository (SQLx implementation)
* CreateInvoice use case with amount and expiration validation
* GetInvoice use case
* ListMerchantInvoices use case
* ExpirePending use case
* POST /api/v1/invoices endpoint
* GET /api/v1/invoices/{id} endpoint
* GET /api/v1/merchants/{merchant_id}/invoices endpoint
* invoices table migration with invoice_status enum

## [0.5.0] - Blockchain Monitoring - 2026-06-13

### Added
* SolanaRpcClient (JSON-RPC over HTTP)
* getSignaturesForAddress integration
* getTransaction integration  
* TransactionWatcher background task (10s interval)
* USDC token balance change detection
* Solana devnet configuration
* SOLANA_RPC_URL and SOLANA_USDC_MINT env vars

## [0.6.0] - Payment Processing - 2026-06-14

### Added
* Payment domain entity with PaymentStatus (Detected, Confirmed, Failed)
* PaymentRepository trait
* PostgresPaymentRepository (SQLx implementation)
* PaymentUseCase with duplicate signature check, invoice matching, amount validation
* payments table migration with payment_status enum
* TransactionWatcher wired to PaymentUseCase — invoices auto-close on payment detection

## [0.7.0] - Webhook System - 2026-06-14

### Added
* webhooks table — merchant endpoint registration
* webhook_events table — event queue with status machine
* webhook_deliveries table — delivery history + retry tracking
* WebhookUseCase: register, create_event, dispatch_pending
* PostgresWebhookRepository
* HMAC-SHA256 request signing (X-Webhook-Signature header)
* Exponential backoff retry (1m, 5m, 30m, 2h, 8h) — max 5 attempts
* Background webhook dispatcher worker (15s interval)
* PaymentUseCase emits invoice.paid webhook event on success

## [0.8.0] - Settlement Engine - 2026-06-14

### Added
* settlements table — per-payment financial record with fee calculation
* settlement_batches table — daily merchant payout grouping
* Settlement domain with state machine (pending → processing → ready_to_pay → paid | failed)
* SettlementUseCase: create, process_pending, batch grouping by merchant
* PostgresSettlementRepository
* Platform fee: 1% (configurable via PLATFORM_FEE_RATE)
* PaymentUseCase auto-creates settlement on payment success
* Background settlement processor worker (60s interval)
* Exponential batch processing — ready for Phase 9 on-chain payout

## [0.9.0] - Admin Platform - 2026-06-14

### Added
* audit_logs table with audit_action enum and indexes
* AuditLogger — async fire-and-forget audit trail
* AdminUseCase — merchant activation, invoice/payment/settlement/webhook oversight
* GET  /api/v1/admin/merchants
* GET  /api/v1/admin/merchants/{id}
* PUT  /api/v1/admin/merchants/{id}/activate
* PUT  /api/v1/admin/merchants/{id}/deactivate
* GET  /api/v1/admin/merchants/{id}/invoices
* GET  /api/v1/admin/merchants/{id}/payments
* GET  /api/v1/admin/merchants/{id}/settlements
* POST /api/v1/admin/settlements/trigger
* GET  /api/v1/admin/webhooks/pending
* POST /api/v1/admin/webhooks/retry
* GET  /api/v1/admin/audit-logs
* GET  /api/v1/admin/merchants/{id}/audit-logs

## [0.10.0] - Production Readiness - 2026-06-14

### Added
* Request ID middleware (x-request-id header, propagated to response)
* API key authentication middleware (x-api-key, merchant validation)
* Protected routes: wallets, invoices require valid API key
* Prometheus metrics endpoint GET /metrics
* OpenTelemetry tracing with OTLP export (optional via OTLP_ENDPOINT env)
* Graceful shutdown on SIGTERM/Ctrl+C with configurable drain timeout
* Rate limiting infrastructure (global, per-IP ready for Phase 10)
* Config: OTLP_ENDPOINT, SHUTDOWN_TIMEOUT_SECS

## [0.11.0] - Public Launch - 2026-06-14

### Added
* Invoice expiry background job (60s interval, auto-expires pending invoices)
* POST /api/v1/webhooks — merchant webhook registration
* GET  /api/v1/merchants/{id}/webhooks — list merchant webhooks
* GET  /docs — Swagger UI
* GET  /api-docs/openapi.json — OpenAPI 3.0 spec
* crypto-gateway-sdk — Rust SDK with GatewayClient
  * create_merchant, get_merchant
  * create_wallet, get_wallet
  * create_invoice, get_invoice
  * register_webhook

## [0.14.0] - Managed Wallet Generation (Custodial) - 2026-06-16

### Added
* `wallet_keys` table — AES-256-GCM encrypted private keys with per-row nonce and key_version
* `WalletVault` service — AES-256-GCM encrypt/decrypt, master key from `WALLET_MASTER_KEY` env, zeroize on use
* `WalletKey` domain entity — never exposed via API or logs
* `WalletKeyRepository` trait + `PostgresWalletKeyRepository` implementation
* `GenerateWalletUseCase` — creates keypair per chain, encrypts key, persists wallet + key atomically
  * Solana: ed25519-dalek keypair → Base58 address
  * EVM (Ethereum/Polygon/Base/BSC): secp256k1 → keccak256 → 0x address
  * Tron: secp256k1 → keccak256 → Base58Check with 0x41 prefix
  * Bitcoin: secp256k1 → SHA256 → RIPEMD160 → P2PKH Base58Check
* `POST /api/v1/wallets/generate` endpoint — returns wallet address, never private key
* New dependencies: `aes-gcm`, `zeroize`, `ed25519-dalek`, `secp256k1`, `bs58`, `sha3`, `ripemd`

### Security
* Private key is zeroized from memory immediately after encryption
* `WALLET_MASTER_KEY` required at startup in production (panic guard)
* Dev fallback (32 zero bytes) only active when `APP_ENV != production`

---

## [0.13.0] - Multi-Network Expansion - 2026-06-16

### Added
* `Blockchain::Bitcoin` and `Asset::Btc` domain variants
* Bitcoin address validation: P2PKH (`1...`), P2SH (`3...`), Bech32/Taproot (`bc1...`)
* `BscAdapter` — BEP-20 token monitoring via EVM-compatible RPC (reuses `EthereumRpcClient`)
* `TronAdapter` — TRC-20 USDT monitoring via TronGrid REST API (`/v1/accounts/{addr}/transactions/trc20`)
* `TronClient` — HTTP client for TronGrid with 10-minute lookback window
* Config: `BSC_RPC_URL`, `BSC_TOKEN_CONTRACT`, `BSC_TOKEN_SYMBOL`, `TRON_RPC_URL`, `TRON_TOKEN_CONTRACT`, `TRON_TOKEN_SYMBOL`
* `Wallet::new()` now accepts explicit `blockchain` and `asset` parameters (removed Solana/USDC hardcode)
* `Wallet::from_generated()` constructor for keypair-derived wallets (skips address validation)
* `POST /api/v1/wallets` now requires `blockchain` and `asset` fields in request body
* `WalletError::UnsupportedBlockchain` and `WalletError::UnsupportedAsset` variants

---

## [0.12.1] - Technical Debt Sprint - 2026-06-16

### Fixed

* `PaymentStatus::from_str` — replaced silent fallback default with `try_from_str` returning `Result<PaymentStatus, PaymentError::InvalidStatus>`. DB corruption now surfaces as an error instead of masking as `Detected`.
* `Payment::new()` — removed hardcoded `blockchain: "solana"` and `asset: "USDC"`. Both fields are now explicit parameters passed from `ProcessPayment` command and sourced from `DetectedPayment`.
* `Payment::confirm()` — restored commented-out method. `PaymentRepository` trait gains `update()`. `PaymentUseCase::process()` now transitions payments to `Confirmed` immediately after detection (TODO: replace with block-confirmation oracle in Phase 13).
* `MultiChainWatcher` — `seen_signatures` now seeded from DB on startup via `find_signatures_by_blockchain()`. Prevents redundant RPC calls after service restart. Financial safety was already guaranteed by DB duplicate-signature check; this fix eliminates unnecessary blockchain RPC load.
* API key generation — replaced UUID concatenation with `rand::rngs::OsRng` + 256-bit entropy. New format: `cgpk_<64-char hex>` (69 chars total).

### Added

* `PaymentError::InvalidStatus(String)` variant
* `PaymentRepository::update()` — persists status + confirmed_at changes
* `PaymentRepository::find_signatures_by_blockchain()` — used for watcher startup seeding
* `DetectedPayment::asset` field on `ChainAdapter` trait
* `rand = "0.8"` dependency
* Unit tests: 10 tests in `payment/domain.rs`, 8 tests in `merchant/domain.rs`

---

  ## [0.12.0] - Multi-Chain - 2026-06-14

### Added
* ChainAdapter trait — blockchain abstraction layer
* SolanaAdapter — refactored from TransactionWatcher
* EthereumRpcClient — JSON-RPC client (eth_getLogs, eth_blockNumber)
* EthereumAdapter — ERC-20 USDC transfer detection
* MultiChainWatcher — parallel multi-chain payment detection
* Ethereum support via ETHEREUM_RPC_URL + ETHEREUM_USDC_CONTRACT env vars
* Polygon/Base compatible (same adapter, different RPC endpoint)

### Removed
* TransactionWatcher — replaced by MultiChainWatcher + SolanaAdapter


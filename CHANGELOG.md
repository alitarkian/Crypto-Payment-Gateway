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


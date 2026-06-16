# Project Memory

This file contains long-term project memory.

Any AI assistant must read this file before generating code.

---

## Project Identity

Project Name:

Crypto Payment Gateway

Primary Blockchain:

Solana

Primary Asset:

USDC

Target Users:

* SaaS Companies
* Online Businesses
* Marketplaces
* Subscription Platforms

---

## Key Decisions

Decision 001

Architecture:

Modular Monolith

Reason:

Faster iteration while maintaining clean boundaries.

---

Decision 002

Blockchain:

Solana First

Reason:

Low fees
High throughput
Strong Rust ecosystem

---

Decision 003

Database:

PostgreSQL

Reason:

Strong consistency
Mature tooling

Database Name: gateway_db
Database User: erpos (shared with existing infrastructure)

---

Decision 004

Cache:

Redis

Reason:

Session storage
Rate limiting
RPC caching

---

Decision 005

Language:

Rust

Reason:

Performance
Security
Reliability

---

Decision 006

Dependency Versions (confirmed working):

* axum = 0.8
* tokio = 1
* tower-http = 0.6
* sqlx = 0.9 (features: runtime-tokio, tls-rustls, postgres, uuid, chrono, migrate)
* redis = 0.25
* config = 0.15
* dotenvy = 0.15
* tracing = 0.1
* tracing-subscriber = 0.3
* uuid = 1
* chrono = 0.4
* thiserror = 2
* anyhow = 1

---

## Infrastructure

Docker services (existing, shared):

* PostgreSQL 16 — localhost:5432
* Redis 7 — localhost:6379
* RabbitMQ 3.13 — localhost:5672

---

## Current Phase

Phase 0 — Foundation

Status: COMPLETE

Completed:

* Project structure
* Config system
* Tracing
* HTTP server
* Health endpoint
* Database connection pool
* Migrations system

Next Phase:

Phase 1 — Identity & Merchant Management

---

## Non-Goals

Not building:

* Exchange
* Trading platform
* DeFi protocol
* NFT marketplace

---

## Product Goal

Become Stripe-like infrastructure for crypto payments.

---

## Development Principles

1. Security over speed.
2. Simplicity over abstraction.
3. Domain-first design.
4. Observable by default.
5. AI-readable architecture.

---

## Future AI Features

Planned:

* Merchant assistant
* Revenue analysis
* Fraud detection
* Financial insights
* Natural language reporting

Not part of MVP.

## Completed Phases

| Phase | Name | Version | Status |
|-------|------|---------|--------|
| 0 | Foundation | 0.1.0 | ✅ COMPLETE |
| 1 | Merchant Management | 0.2.0 | ✅ COMPLETE |
| 2 | Wallet Infrastructure | 0.3.0 | ✅ COMPLETE |
| 3 | Invoice Engine | 0.4.0 | ✅ COMPLETE |
| 4 | Blockchain Monitoring | 0.5.0 | ✅ COMPLETE |
| 5 | Payment Processing | 0.6.0 | ✅ COMPLETE |
| 6 | Webhook System | 0.7.0 | ✅ COMPLETE |
| 7 | Settlement Engine | 0.8.0 | ✅ COMPLETE |
| 8 | Admin Platform | 0.9.0 | ✅ COMPLETE |
| 9 | Production Readiness | 0.10.0 | ✅ COMPLETE |
| 10 | Public Launch | 0.11.0 | ✅ COMPLETE |
| 11 | Multi-Chain (Eth/Polygon/Base) | 0.12.0 | ✅ COMPLETE |
| — | Technical Debt Sprint | 0.12.1 | ✅ COMPLETE |
| 12 | Multi-Network Expansion (BSC/Tron/Bitcoin) | 0.13.0 | ✅ COMPLETE |
| 13 | Managed Wallet Generation (Custodial) | 0.14.0 | ✅ COMPLETE |
| — | Compilation Fixes (Rust 2024 + Deps) | 0.14.1 | ✅ COMPLETE |
| — | Docker Deployment | 0.15.0 | ✅ COMPLETE |

---

## Current Phase

Phase 14 — Next (TBD)

Status: NOT STARTED

Next deliverables:
* Merchant Assistant
* Revenue Analytics
* Fraud Detection
* Natural Language Reporting

---

## Known Patterns & Invariants

### Payment Flow
1. `MultiChainWatcher` detects on-chain tx → builds `ProcessPayment` command
2. `PaymentUseCase::process()` checks duplicate signature in DB
3. Saves payment with status `Detected`, immediately calls `confirm()` → status `Confirmed`
4. Marks invoice as `Paid`, creates `Settlement`, emits `invoice.paid` webhook event

### Blockchain field propagation
`DetectedPayment` carries both `blockchain` (e.g. `"solana"`, `"ethereum"`) and `asset` (e.g. `"USDC"`).
These are passed through `ProcessPayment` → `Payment::new()` and stored in DB.
Do NOT hardcode blockchain or asset in domain constructors.

### API Key format
Format: `cgpk_<64-char lowercase hex>` (256-bit OsRng entropy, total 69 chars)
Old UUID-based format (`mk_...`) is deprecated — existing keys in DB remain valid.

### seen_signatures seeding
`MultiChainWatcher` seeds `seen_signatures` from DB on startup via
`PaymentRepository::find_signatures_by_blockchain()`.
This prevents redundant RPC calls after service restart.
Duplicate-payment protection is enforced at DB level regardless.

### Wallet Types
Two types of wallets exist:
1. **Watch Wallet** (`POST /api/v1/wallets`) — merchant provides their own address. System monitors it.
2. **Managed Wallet** (`POST /api/v1/wallets/generate`) — system generates keypair, encrypts private key, returns only the address.

### Managed Wallet Key Storage
- Private keys stored in `wallet_keys` table as AES-256-GCM ciphertext
- Master key from `WALLET_MASTER_KEY` env var (64-char hex = 32 bytes)
- `key_version` column enables future key rotation
- `WalletVault` is the ONLY service allowed to decrypt keys
- Private key is NEVER returned in API responses, NEVER logged
- In production, missing `WALLET_MASTER_KEY` causes panic at startup

### Supported Blockchains + Key Types
| Blockchain | Key Type | Address Format |
|------------|----------|---------------|
| Solana | ed25519 | Base58 (32-44 chars) |
| Ethereum / Polygon / Base / BSC | secp256k1 | `0x` + 20-byte hex |
| Tron | secp256k1 | Base58Check, starts with `T`, 34 chars |
| Bitcoin | secp256k1 | P2PKH Base58Check, starts with `1` |

### Supported Assets
| Asset | Chains | Decimals |
|-------|--------|----------|
| USDC | Solana, Ethereum, Polygon, Base | 6 |
| USDT | Tron (TRC-20), BSC (BEP-20) | 6 |
| SOL | Solana | 9 |
| ETH | Ethereum | 18 |
| BNB | BSC | 18 |
| TRX | Tron | 6 |
| BTC | Bitcoin | 8 |

### Active Chain Adapters
| Chain | Adapter | Config Env Vars |
|-------|---------|----------------|
| Solana | SolanaAdapter | `SOLANA_RPC_URL`, `SOLANA_USDC_MINT` |
| Ethereum | EthereumAdapter | `ETHEREUM_RPC_URL`, `ETHEREUM_USDC_CONTRACT` |
| BSC | BscAdapter | `BSC_RPC_URL`, `BSC_TOKEN_CONTRACT`, `BSC_TOKEN_SYMBOL` |
| Tron | TronAdapter | `TRON_RPC_URL`, `TRON_TOKEN_CONTRACT`, `TRON_TOKEN_SYMBOL` |

All adapters are optional — only enabled when env vars are set.

### Docker Setup
- Named network: `crypto_gateway_net` (defined in docker-compose, other projects join as external)
- App container name: `crypto_gateway`, port `8080`
- Build requires `.sqlx/` cache — run `cargo sqlx prepare` before `docker build`
- Env file: `.env.docker` (uses service names as hostnames, NOT localhost)
- Migrations run automatically at startup via `sqlx::migrate!` embedded in binary
- Credentials inside Docker: `erpos:erpos` for both PostgreSQL and RabbitMQ

### How another project connects
```yaml
# in other project's docker-compose.yml:
networks:
  crypto_gateway_net:
    external: true

services:
  my_service:
    networks:
      - crypto_gateway_net
    # call gateway at: http://crypto_gateway:8080
```

### Rust 2024 Edition Gotchas (confirmed fixes)
- `std::env::set_var` is unsafe → wrap in `unsafe {}`
- Temporaries in array literals (`.to_string()` reference) cause type inference failures → hoist to variable first
- `aes-gcm` nonce: use `Aes256Gcm::generate_nonce(&mut OsRng)` (not deprecated `from_slice`)
- `reqwest 0.13` `.query()` method not always available → build URL query string manually
- `secp256k1 v0.29` feature is `hashes` not `bitcoin_hashes`

### TODO markers
* `TODO(phase-13)` — block-confirmation oracle: replace immediate `confirm()` call
  in `PaymentUseCase::process()` with N-confirmation wait before confirming.
* `TODO(future)` — migrate `WalletVault` master key to AWS KMS or HashiCorp Vault for HSM-grade security.
* `TODO(future)` — Bitcoin monitoring adapter (Blockstream API or Bitcoin Core RPC).


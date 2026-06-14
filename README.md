# Crypto Payment Gateway

> Production-grade crypto payment infrastructure built with Rust — the Stripe for crypto payments.

[![Rust](https://img.shields.io/badge/rust-1.96-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

---

## Overview

Crypto Payment Gateway is a high-performance, production-ready payment infrastructure that enables merchants to accept blockchain payments with minimal integration complexity. Built entirely in Rust, it provides the reliability and security required for financial systems at scale.

**Primary focus:** Solana + USDC, with Ethereum/Polygon/Base support via the multi-chain adapter layer.

---

## Features

- **Merchant Management** — onboarding, API key issuance, activation/deactivation
- **Wallet Infrastructure** — Solana and Ethereum wallet registration and monitoring
- **Invoice Engine** — create payable invoices with expiration, status tracking, and auto-close
- **Blockchain Monitoring** — real-time on-chain payment detection via MultiChainWatcher
- **Payment Processing** — duplicate protection, amount validation, invoice reconciliation
- **Webhook System** — HMAC-SHA256 signed events with exponential backoff retry
- **Settlement Engine** — automated per-payment fee calculation and daily batch grouping
- **Admin Platform** — merchant oversight, manual triggers, full audit trail
- **Multi-Chain** — pluggable ChainAdapter for Solana, Ethereum, Polygon, Base
- **Observability** — Prometheus metrics, OpenTelemetry tracing, structured logging
- **Security** — API key auth, request ID propagation, graceful shutdown
- **SDK** — `crypto-gateway-sdk` Rust crate for easy integration
- **OpenAPI** — Swagger UI at `/docs`, JSON spec at `/api-docs/openapi.json`

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2024) |
| HTTP Framework | Axum 0.8 |
| Async Runtime | Tokio |
| Database | PostgreSQL 16 + SQLx 0.9 |
| Cache | Redis 7 |
| Message Broker | RabbitMQ 3.13 |
| Blockchain | Solana RPC, Ethereum JSON-RPC |
| Observability | Prometheus, OpenTelemetry (OTLP) |
| Infrastructure | Docker, Docker Compose |

---

## Architecture

The system follows a **Modular Monolith** architecture with clean boundaries between modules. Each module owns its domain, use cases, repository interface, and HTTP handlers — independently extractable to microservices when justified.

```
src/
├── config/              # Environment-based configuration
├── infrastructure/      # DB, blockchain clients, repository implementations
│   └── blockchain/
│       ├── adapters/    # SolanaAdapter, EthereumAdapter
│       ├── chain_adapter.rs   # ChainAdapter trait
│       └── multi_chain_watcher.rs
├── middleware/          # Auth, request ID, rate limiting
├── modules/
│   ├── merchant/        # Merchant domain + API
│   ├── wallet/          # Wallet domain + API
│   ├── invoice/         # Invoice domain + API
│   ├── payment/         # Payment processing
│   ├── settlement/      # Settlement engine
│   ├── webhook/         # Webhook delivery system
│   └── admin/           # Admin platform + audit logs
├── observability/       # Metrics, tracing
└── openapi/             # Swagger UI + OpenAPI spec
sdk/                     # crypto-gateway-sdk crate
```

---

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker and Docker Compose
- PostgreSQL 16
- Redis 7

### 1. Clone and configure

```bash
git clone https://github.com/your-org/crypto-payment-gateway
cd crypto-payment-gateway

cat > .env << 'EOF'
APP_ENV=development
APP_HOST=0.0.0.0
APP_PORT=8080
DATABASE_URL=postgres://gateway:gateway@localhost:5432/gateway_db
REDIS_URL=redis://localhost:6379
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_USDC_MINT=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
SHUTDOWN_TIMEOUT_SECS=30

# Optional: Ethereum support
# ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_KEY
# ETHEREUM_USDC_CONTRACT=0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Optional: OpenTelemetry
# OTLP_ENDPOINT=http://localhost:4317
EOF
```

### 2. Start infrastructure

```bash
docker compose up -d
```

### 3. Run

```bash
cargo run
```

### 4. Verify

```bash
curl http://localhost:8080/health
# {"status":"ok","version":"0.1.0"}
```

---

## API Reference

Full interactive documentation available at **http://localhost:8080/docs**

### Authentication

Protected endpoints require an `x-api-key` header with the merchant's API key.

```
x-api-key: mk_c069b959f73d...
```

### Endpoints

#### Merchants

```
POST   /api/v1/merchants              Create merchant
GET    /api/v1/merchants/{id}         Get merchant
```

#### Wallets

```
POST   /api/v1/wallets                Create wallet          🔒
GET    /api/v1/wallets/{id}           Get wallet             🔒
GET    /api/v1/merchants/{id}/wallets List merchant wallets  🔒
```

#### Invoices

```
POST   /api/v1/invoices               Create invoice         🔒
GET    /api/v1/invoices/{id}          Get invoice            🔒
GET    /api/v1/merchants/{id}/invoices List invoices         🔒
```

#### Webhooks

```
POST   /api/v1/webhooks               Register endpoint      🔒
GET    /api/v1/merchants/{id}/webhooks List webhooks         🔒
```

#### Admin

```
GET    /api/v1/admin/merchants                    List all merchants
GET    /api/v1/admin/merchants/{id}               Get merchant
PUT    /api/v1/admin/merchants/{id}/activate      Activate
PUT    /api/v1/admin/merchants/{id}/deactivate    Deactivate
GET    /api/v1/admin/merchants/{id}/invoices      Merchant invoices
GET    /api/v1/admin/merchants/{id}/payments      Merchant payments
GET    /api/v1/admin/merchants/{id}/settlements   Merchant settlements
POST   /api/v1/admin/settlements/trigger          Trigger settlement
GET    /api/v1/admin/webhooks/pending             Pending events
POST   /api/v1/admin/webhooks/retry               Retry delivery
GET    /api/v1/admin/audit-logs                   Platform audit log
GET    /api/v1/admin/merchants/{id}/audit-logs    Merchant audit log
```

#### System

```
GET    /health                         Health check
GET    /metrics                        Prometheus metrics
GET    /docs                           Swagger UI
GET    /api-docs/openapi.json          OpenAPI spec
```

---

## Payment Flow

```
Merchant creates Invoice
        │
        ▼
MultiChainWatcher polls wallet address every 10s
        │
        ▼
Payment detected on-chain (Solana / Ethereum)
        │
        ▼
PaymentUseCase: dedup check → invoice match → amount validation
        │
        ├──► Invoice marked Paid
        ├──► Settlement created (1% platform fee)
        └──► Webhook fired (invoice.paid event)
```

---

## Webhook Events

All webhook requests are signed with HMAC-SHA256:

```
X-Webhook-Signature: <hex>
X-Webhook-Event: invoice.paid
```

Retry schedule (exponential backoff): 1m → 5m → 30m → 2h → 8h (max 5 attempts)

Supported events:

- `invoice.paid`
- `invoice.expired`
- `payment.detected`

---

## Settlement Engine

Payments are automatically settled:

1. Payment confirmed → `Settlement` record created with 1% fee
2. Background processor (every 60s) groups pending settlements into daily batches per merchant
3. Batch status: `open` → `processing` → `ready_to_pay` (on-chain transfer in Phase 9+)

Fee rate is configurable via `PLATFORM_FEE_RATE` (default: 1%).

---

## Multi-Chain Support

The `ChainAdapter` trait abstracts all blockchain-specific logic:

```rust
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    fn chain_name(&self) -> &str;
    async fn detect_payments(
        &self,
        wallet_address: &str,
        seen_signatures: &mut HashSet<String>,
    ) -> anyhow::Result<Vec<DetectedPayment>>;
}
```

Currently supported:

| Chain | Status | Notes |
|---|---|---|
| Solana | ✅ Production | USDC via SPL Token |
| Ethereum | ✅ Available | ERC-20 USDC via eth_getLogs |
| Polygon | ✅ Compatible | Same adapter, different RPC |
| Base | ✅ Compatible | Same adapter, different RPC |

---

## SDK Usage

```toml
[dependencies]
crypto-gateway-sdk = { path = "./sdk" }
```

```rust
use crypto_gateway_sdk::{GatewayClient, CreateMerchantRequest, CreateInvoiceRequest};

let client = GatewayClient::new("https://your-gateway.com")
    .with_api_key("mk_your_api_key");

// Create invoice
let invoice = client.create_invoice(CreateInvoiceRequest {
    merchant_id: merchant_id,
    wallet_id: wallet_id,
    amount: Decimal::new(1000, 2), // 10.00 USDC
    description: Some("Order #1234".into()),
    expires_at: Utc::now() + Duration::hours(24),
}).await?;

println!("Pay {} USDC to {}", invoice.amount, wallet_address);
```

---

## Observability

### Prometheus Metrics

Available at `GET /metrics`:

- `http_requests_total` — labeled by method, path, status
- `http_request_duration_seconds` — histogram
- `payments_total` — labeled by status
- `active_invoices` — gauge

### OpenTelemetry Tracing

Set `OTLP_ENDPOINT` to export traces to Jaeger, Grafana Tempo, or any OTLP-compatible backend:

```bash
OTLP_ENDPOINT=http://localhost:4317 cargo run
```

### Structured Logging

All logs are structured and include `request_id` for full request tracing:

```json
{"timestamp":"2026-06-14T07:19:19Z","level":"INFO","message":"Payment processed","payment_id":"...","invoice_id":"...","amount":"10.00"}
```

---

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `APP_ENV` | ✅ | — | `development` / `production` |
| `APP_HOST` | ✅ | — | Bind address |
| `APP_PORT` | ✅ | — | HTTP port |
| `DATABASE_URL` | ✅ | — | PostgreSQL connection string |
| `REDIS_URL` | ✅ | — | Redis connection string |
| `SOLANA_RPC_URL` | ✅ | — | Solana RPC endpoint |
| `SOLANA_USDC_MINT` | ✅ | — | USDC mint address |
| `ETHEREUM_RPC_URL` | ☐ | — | Ethereum RPC (enables ETH support) |
| `ETHEREUM_USDC_CONTRACT` | ☐ | — | USDC contract address on Ethereum |
| `OTLP_ENDPOINT` | ☐ | — | OpenTelemetry collector endpoint |
| `SHUTDOWN_TIMEOUT_SECS` | ☐ | `30` | Graceful shutdown drain timeout |

---

## Development

```bash
# Run with hot reload
cargo watch -x run

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Check without building
cargo check
```

### Database Migrations

Migrations run automatically on startup via SQLx. To add a new migration:

```bash
sqlx migrate add <migration_name>
# Edit migrations/<timestamp>_<name>.sql
```

---

## Roadmap

| Phase | Status | Description |
|---|---|---|
| 0 — Foundation | ✅ Complete | Rust + Axum + PostgreSQL + Redis |
| 1 — Merchant Management | ✅ Complete | Onboarding + API keys |
| 2 — Wallet Infrastructure | ✅ Complete | Wallet creation + monitoring |
| 3 — Invoice Engine | ✅ Complete | Payable invoices + expiration |
| 4 — Blockchain Monitoring | ✅ Complete | Solana RPC integration |
| 5 — Payment Processing | ✅ Complete | Match + reconcile + dedup |
| 6 — Webhook System | ✅ Complete | Signed events + retry queue |
| 7 — Settlement Engine | ✅ Complete | Fee calculation + daily batches |
| 8 — Admin Platform | ✅ Complete | Oversight + audit logs |
| 9 — Production Readiness | ✅ Complete | Metrics + tracing + auth |
| 10 — Public Launch | ✅ Complete | SDK + docs + webhook API |
| 11 — Multi-Chain | ✅ Complete | Ethereum + Polygon + Base |
| 12 — AI Layer | 🔄 Planned | Merchant assistant + fraud detection |

---

## Security

- All secrets loaded from environment variables — never hardcoded
- API key authentication on all merchant-facing routes
- HMAC-SHA256 webhook signatures
- Audit log for all admin actions
- Input validation at domain layer
- No business logic in HTTP handlers or repositories
- Principle of least privilege throughout

---

## License

MIT — see [LICENSE](LICENSE)

---

## Contributing

This project follows strict production-first principles. See [AI_RULES.md](AI_RULES.md) for development guidelines.

Branch naming: `feature/*`, `fix/*`, `refactor/*`, `docs/*`

Commit format: `feat(module): description`

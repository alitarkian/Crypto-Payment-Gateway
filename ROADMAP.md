# ROADMAP.md

# Product Roadmap

Project:

Crypto Payment Gateway

Status:

Active

---

# Phase 0

Foundation

Status:

Current

Goal:

Establish architecture and infrastructure.

Deliverables:

* Rust setup
* Axum
* PostgreSQL
* Redis
* Docker
* Configuration system
* Logging
* Health checks

Exit Criteria:

System boots successfully.

---

# Phase 1

Identity & Merchant Management

Goal:

Enable merchant onboarding.

Deliverables:

* Merchant entity
* Merchant API
* API keys
* Authentication
* Authorization

Exit Criteria:

Merchant can create and manage account.

---

# Phase 2

Wallet Infrastructure

Goal:

Enable wallet creation and monitoring.

Deliverables:

* Wallet creation
* Wallet storage
* Address validation
* Balance tracking

Exit Criteria:

Merchant owns active wallet.

---

# Phase 3

Invoice Engine

Goal:

Create payable invoices.

Deliverables:

* Invoice generation
* Expiration logic
* Status management

Statuses:

Draft
Pending
Paid
Expired
Cancelled

Exit Criteria:

Merchant can generate invoice.

---

# Phase 4

Blockchain Monitoring

Goal:

Detect on-chain payments.

Deliverables:

* Solana RPC integration
* Transaction watcher
* Confirmation engine

Exit Criteria:

Incoming payments detected automatically.

---

# Phase 5

Payment Processing

Goal:

Connect invoices to transactions.

Deliverables:

* Payment matching
* Reconciliation
* Duplicate protection

Exit Criteria:

Invoices close automatically after payment.

---

# Phase 6

Webhook System

Goal:

Notify external systems.

Deliverables:

* Webhook creation
* Retry queue
* Signature verification

Exit Criteria:

External applications receive events.

---

# Phase 7

Settlement Engine

Goal:

Automated merchant payouts.

Deliverables:

* Settlement workflow
* Fee calculation
* Transfer execution

Exit Criteria:

Funds distributed automatically.

---

# Phase 8

Admin Platform

Goal:

Operational control.

Deliverables:

* Merchant management
* Audit logs
* Platform analytics

Exit Criteria:

Operations manageable without database access.

---

# Phase 9

Production Readiness

Goal:

Enterprise deployment.

Deliverables:

* Metrics
* Tracing
* Alerting
* Security hardening
* Backups

Exit Criteria:

Production launch approved.

---

# Phase 10

Public Launch

Goal:

Acquire first paying customers.

Deliverables:

* Landing page
* Documentation
* SDK
* Customer onboarding

Exit Criteria:

First real merchant onboarded.

---

# Phase 11

Multi-Chain

Goal:

Support additional chains.

Deliverables:

* Ethereum
* Polygon
* Base

Exit Criteria:

Multi-chain payments operational.

---

# Phase 12

AI Layer

Goal:

Financial intelligence platform.

Deliverables:

* Merchant Assistant
* Revenue Analytics
* Fraud Detection
* Natural Language Reporting

Exit Criteria:

AI features used by active merchants.

---

# Golden Rule

Never start a new phase before the current phase exit criteria are fully satisfied.

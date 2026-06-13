# ARCHITECTURE_DECISIONS.md

# Architecture Decision Records (ADR)

This document records all major architectural decisions.

Purpose:

* Preserve decision history
* Explain rationale
* Help future developers
* Help AI assistants maintain context
* Avoid repeating past discussions

Every important architectural decision MUST be recorded here.

---

# ADR-000

Title:

Project Initialization

Status:

Accepted

Date:

2026-06-13

Decision:

Create a production-grade crypto payment gateway using Rust and Solana.

Context:

Project aims to become payment infrastructure rather than a crypto wallet application.

Consequences:

Architecture must prioritize scalability, reliability, security, and maintainability.

---

# ADR-001

Title:

Modular Monolith Architecture

Status:

Accepted

Date:

2026-06-13

Decision:

Start as Modular Monolith.

Context:

Premature microservices create operational complexity.

Current team size and product maturity do not justify distributed services.

Consequences:

Pros:

* Faster development
* Easier testing
* Simpler deployments
* Lower operational cost

Cons:

* Future extraction work may be required

Migration Strategy:

Modules must remain independently extractable.

---

# ADR-002

Title:

Rust as Core Language

Status:

Accepted

Date:

2026-06-13

Decision:

Use Rust for all backend services.

Context:

System requires:

* Performance
* Reliability
* Memory safety
* Concurrency

Consequences:

Higher learning curve.

Lower runtime risks.

---

# ADR-003

Title:

Solana First Strategy

Status:

Accepted

Date:

2026-06-13

Decision:

Support Solana before any other blockchain.

Context:

* Native Rust ecosystem
* Low transaction fees
* High throughput

Consequences:

Initial blockchain abstraction layer may be Solana-centric.

Future chains will require adapter interfaces.

---

# ADR-004

Title:

USDC First Settlement Asset

Status:

Accepted

Date:

2026-06-13

Decision:

Use USDC as primary payment asset.

Context:

Merchants need stable value.

Volatile assets create accounting challenges.

Consequences:

Payment workflows focus on stablecoin operations.

Future support for additional assets remains possible.

---

# ADR-005

Title:

PostgreSQL as System Database

Status:

Accepted

Date:

2026-06-13

Decision:

Use PostgreSQL.

Context:

Strong consistency and mature ecosystem.

Consequences:

Avoid database-specific features that prevent portability unless justified.

---

# ADR-006

Title:

Redis as Infrastructure Cache

Status:

Accepted

Date:

2026-06-13

Decision:

Use Redis.

Context:

Needed for:

* Caching
* Rate limiting
* Session storage
* Temporary state

Consequences:

Redis is not a source of truth.

PostgreSQL remains authoritative.

---

# ADR-007

Title:

Domain-Driven Design

Status:

Accepted

Date:

2026-06-13

Decision:

Follow DDD principles.

Context:

Payment systems contain complex business rules.

Consequences:

Domain logic must remain independent from infrastructure.

Forbidden:

* SQL in domain
* HTTP in domain
* Blockchain SDK in domain

---

# ADR-008

Title:

Observability First

Status:

Accepted

Date:

2026-06-13

Decision:

All services must emit logs, traces, and metrics.

Context:

Financial systems require visibility and auditability.

Consequences:

No production feature may be merged without observability support.

---

# ADR-009

Title:

AI Integration Deferred

Status:

Accepted

Date:

2026-06-13

Decision:

Do not implement AI in MVP.

Context:

Core payment infrastructure must stabilize first.

Consequences:

AI capabilities are future extensions.

Future Areas:

* Merchant assistant
* Revenue analytics
* Fraud analysis
* Natural language reporting

---

# ADR Template

For every new ADR:

---

# ADR-XXX

Title:

Status:

Proposed | Accepted | Deprecated | Superseded

Date:

YYYY-MM-DD

Decision:

Context:

Alternatives Considered:

Consequences:

Migration Notes:

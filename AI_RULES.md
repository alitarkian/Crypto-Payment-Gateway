# AI_RULES.md

# AI Development Rules

This file is mandatory reading for any AI assistant contributing to this project.

Applies to:

* ChatGPT
* Claude
* Gemini
* Cursor
* Copilot
* Future AI systems

Failure to follow these rules is considered a project violation.

---

# Project Mission

Build a production-grade crypto payment gateway.

Not a tutorial.

Not a prototype.

Not a code experiment.

Every code contribution must move the project toward production readiness.

---

# Required Context Files

Before generating code, AI must read:

1. PROJECT_STRUCTURE.md
2. MEMORY.md
3. ARCHITECTURE_DECISIONS.md
4. CHANGELOG.md
5. AI_RULES.md

If any file is unavailable, AI must explicitly mention the missing context.

---

# Code Quality Standards

Mandatory:

* Clean Architecture
* DDD principles
* SOLID principles
* Rustfmt compliant
* Clippy clean
* No warnings
* No dead code

Forbidden:

* Quick fixes
* Temporary hacks
* TODO without issue reference
* Commented-out code
* Hidden side effects

---

# Production First Rule

Never generate code intended only for demo purposes.

Every implementation must be production-oriented.

Avoid:

* Mock business logic
* Hardcoded credentials
* Fake security mechanisms

---

# Security Rules

Mandatory:

* Input validation
* Output sanitization
* Secrets isolation
* Audit logging
* Principle of least privilege

Forbidden:

* Plain-text secrets
* Hardcoded private keys
* Hardcoded API keys
* Hardcoded passwords

Immediate rejection if detected.

---

# Domain Rules

Business logic belongs only in Domain or Application layers.

Forbidden:

* Business logic inside handlers
* Business logic inside repositories
* Business logic inside middleware

---

# Database Rules

Use:

* SQLx
* PostgreSQL

Mandatory:

* Explicit migrations
* Transaction safety
* Index awareness

Forbidden:

* Raw SQL inside handlers
* Silent migration changes

---

# Blockchain Rules

All blockchain operations must be abstracted.

Never couple domain logic directly to Solana SDK.

Use adapters/interfaces.

Goal:

Future support for:

* Solana
* Ethereum
* Polygon
* Base

without domain changes.

---

# API Rules

Use:

* REST first

Future:

* gRPC
* WebSocket

Mandatory:

* Versioned APIs

Example:

/api/v1/...

Never expose internal structures directly.

---

# Logging Rules

Every critical action must be logged.

Examples:

* Merchant creation
* Invoice creation
* Payment detection
* Settlement execution

Forbidden:

* Logging secrets
* Logging private keys

---

# Testing Rules

Required before merge:

Unit Tests

Integration Tests

Critical business rules must always have tests.

Forbidden:

Code without tests for critical workflows.

---

# Error Handling Rules

Never use panic for recoverable errors.

Prefer:

Result<T, E>

Custom domain errors.

Errors must be actionable.

Bad:

"Something went wrong"

Good:

"Invoice already settled"

---

# Dependency Rules

Before adding dependency:

Ask:

1. Can existing code solve this?
2. Is dependency actively maintained?
3. Is dependency security-reviewed?
4. Is dependency really necessary?

Avoid dependency bloat.

---

# Performance Rules

Optimize only after measurement.

Forbidden:

Premature optimization.

Required:

* Tracing
* Metrics
* Profiling

before major optimization work.

---

# Documentation Rules

Every major feature must update:

* CHANGELOG.md
* Relevant ADR
* README

If architecture changes:

Create a new ADR.

---

# Git Rules

Branch Naming:

feature/*
fix/*
refactor/*
docs/*

Commit Format:

feat:
fix:
refactor:
docs:
test:
chore:

Examples:

feat(wallet): add wallet creation endpoint

fix(invoice): prevent duplicate settlement

---

# AI Output Rules

When generating code:

Always provide:

1. Purpose
2. Files affected
3. Risks
4. Testing strategy

Never generate code without explaining impact.

---

# AI Memory Update Rule

Whenever a major decision changes:

Update:

* MEMORY.md
* ARCHITECTURE_DECISIONS.md

Whenever a feature changes:

Update:

* CHANGELOG.md

Context drift is not allowed.

---

# Definition Of Done

A feature is considered complete only if:

* Code implemented
* Tests passing
* Lint passing
* Documentation updated
* Changelog updated
* ADR updated if needed

Otherwise feature remains incomplete.

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

## Current Phase
Phase 1 — Merchant Management
Status: COMPLETE

Next Phase:
Phase 2 — Wallet Infrastructure

## Current Phase
Phase 5 — Payment Processing
Status: COMPLETE

Next Phase:
Phase 6 — Webhook System

## Current Phase
Phase 6 — Webhook System
Status: COMPLETE

Next Phase:
Phase 7 — Settlement Engine

## Current Phase
Phase 7 — Settlement Engine
Status: COMPLETE

Next Phase:
Phase 8 — Admin Platform

## Current Phase
Phase 8 — Admin Platform
Status: COMPLETE

Next Phase:
Phase 9 — Production Readiness


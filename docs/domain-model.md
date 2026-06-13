# docs/domain-model.md

# Domain Model

Project:

Crypto Payment Gateway

Version:

1.0

Status:

Active

---

# Purpose

This document defines the core business entities and relationships.

This is the source of truth for the business domain.

All implementation must follow this model.

If implementation conflicts with this document:

Implementation is wrong.

---

# Ubiquitous Language

Merchant

Business using the platform.

Wallet

Blockchain address controlled by the platform.

Invoice

Payment request issued to a customer.

Payment

On-chain transaction received.

Settlement

Transfer of collected funds.

Webhook

Event delivered to external systems.

Ledger

Financial audit record.

---

# Core Aggregate Map

Merchant

├── Wallet

├── Invoice

├── Webhook

└── Settlement

Invoice

└── Payment

Payment

└── Ledger Entries

---

# Entity: Merchant

Represents a customer of the platform.

---

Merchant

id

merchant_code

name

email

status

created_at

updated_at

---

Status

Pending

Active

Suspended

Closed

---

Business Rules

Merchant code must be unique.

Merchant email must be unique.

Closed merchants cannot create invoices.

Suspended merchants cannot receive settlements.

---

# Entity: ApiKey

Represents merchant authentication.

---

ApiKey

id

merchant_id

key_hash

name

status

expires_at

created_at

---

Status

Active

Revoked

Expired

---

Rules

Raw key must never be stored.

Only hashes may be persisted.

---

# Entity: Wallet

Represents blockchain address ownership.

---

Wallet

id

merchant_id

blockchain

address

status

created_at

updated_at

---

Status

Active

Disabled

Archived

---

Rules

Wallet belongs to exactly one merchant.

Address uniqueness enforced.

Wallet cannot be deleted.

Only archived.

---

# Entity: Invoice

Represents payment request.

---

Invoice

id

merchant_id

invoice_number

currency

amount

description

status

expires_at

created_at

updated_at

---

Status

Draft

Pending

Paid

Expired

Cancelled

---

Rules

Invoice number unique per merchant.

Paid invoices cannot be modified.

Expired invoices cannot become paid.

Invoice amount must be positive.

---

# Entity: Payment

Represents blockchain payment.

---

Payment

id

merchant_id

invoice_id

wallet_id

transaction_hash

amount

currency

status

confirmed_at

created_at

---

Status

Detected

Confirmed

Failed

Reversed

---

Rules

Transaction hash unique.

Payment immutable after confirmation.

Payment amount cannot be negative.

---

# Entity: Settlement

Represents merchant payout.

---

Settlement

id

merchant_id

settlement_number

amount

currency

status

executed_at

created_at

---

Status

Pending

Processing

Completed

Failed

Cancelled

---

Rules

Settlement amount must be positive.

Completed settlements are immutable.

Cancelled settlements cannot be resumed.

---

# Entity: Webhook

Represents merchant event endpoint.

---

Webhook

id

merchant_id

url

secret

status

created_at

---

Status

Active

Disabled

---

Rules

Secret must be encrypted.

URL validation required.

HTTPS required in production.

---

# Entity: LedgerEntry

Financial audit trail.

Immutable.

---

LedgerEntry

id

merchant_id

reference_type

reference_id

entry_type

amount

currency

created_at

---

Entry Types

PaymentReceived

SettlementCreated

SettlementCompleted

RefundCreated

FeeCollected

---

Rules

Ledger entries cannot be updated.

Ledger entries cannot be deleted.

Append-only model.

---

# Value Objects

Money

amount

currency

---

BlockchainAddress

value

---

EmailAddress

value

---

InvoiceNumber

value

---

SettlementNumber

value

---

TransactionHash

value

---

# Domain Events

MerchantCreated

MerchantActivated

WalletCreated

InvoiceCreated

InvoiceExpired

PaymentDetected

PaymentConfirmed

SettlementCreated

SettlementCompleted

WebhookDelivered

WebhookFailed

---

# Invariants

System-wide rules.

1.

Transaction hash must be unique.

2.

Wallet address must be unique.

3.

Ledger entries are immutable.

4.

Payments are immutable after confirmation.

5.

Settlements are immutable after completion.

6.

Invoices cannot be modified after payment.

7.

Every payment must produce ledger entries.

8.

Every settlement must produce ledger entries.

---

# Future Entities

Not in MVP.

Planned:

Customer

Subscription

Refund

Dispute

ExchangeRate

FraudCase

AIInsight

MerchantAssistant

RevenueForecast

---

# Aggregate Boundaries

Merchant Aggregate

Merchant

ApiKey

Webhook

---

Wallet Aggregate

Wallet

---

Invoice Aggregate

Invoice

Payment

---

Settlement Aggregate

Settlement

---

Ledger Aggregate

LedgerEntry

---

No aggregate may directly modify another aggregate's internal state.

Cross-aggregate communication must happen through domain events.

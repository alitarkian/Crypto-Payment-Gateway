# docs/event-storming.md

# Event Storming

Project:

Crypto Payment Gateway

Version:

1.0

Status:

Active

Purpose:

Describe business workflows using:

* Actors
* Commands
* Events
* Policies
* Read Models
* External Systems

This document is the source of truth for workflow design.

---

# Legend

Actor

A user or system initiating actions.

Command

An intent to perform an action.

Event

A fact that already happened.

Policy

Business automation triggered by events.

Read Model

Data optimized for querying.

External System

Outside dependency.

---

# Actors

Merchant

Platform Admin

Customer

Blockchain Monitor

Settlement Processor

Webhook Dispatcher

System Scheduler

---

# Workflow 1

Merchant Registration

Actor:

Merchant

Command:

RegisterMerchant

Event:

MerchantRegistered

Policy:

CreateDefaultMerchantSettings

Event:

MerchantInitialized

Read Models:

MerchantDashboard

MerchantProfile

---

Flow

Merchant
→ RegisterMerchant

MerchantRegistered

MerchantInitialized

---

# Workflow 2

API Key Creation

Actor:

Merchant

Command:

CreateApiKey

Event:

ApiKeyCreated

Policy:

StoreHashedKey

Event:

ApiKeyActivated

Read Models:

ApiKeyList

---

Flow

Merchant
→ CreateApiKey

ApiKeyCreated

ApiKeyActivated

---

# Workflow 3

Wallet Creation

Actor:

Merchant

Command:

CreateWallet

External System:

Solana Wallet Generator

Event:

WalletCreated

Policy:

RegisterWalletMonitoring

Event:

WalletMonitoringEnabled

Read Models:

WalletList

WalletDetails

---

Flow

Merchant
→ CreateWallet

WalletCreated

WalletMonitoringEnabled

---

# Workflow 4

Invoice Creation

Actor:

Merchant

Command:

CreateInvoice

Event:

InvoiceCreated

Policy:

ScheduleExpirationCheck

Event:

InvoiceActivated

Read Models:

InvoiceList

InvoiceDetails

---

Flow

Merchant
→ CreateInvoice

InvoiceCreated

InvoiceActivated

---

# Workflow 5

Invoice Expiration

Actor:

System Scheduler

Command:

ExpireInvoice

Event:

InvoiceExpired

Policy:

NotifyMerchant

Event:

ExpirationNotificationSent

Read Models:

InvoiceStatus

---

Flow

Scheduler
→ ExpireInvoice

InvoiceExpired

ExpirationNotificationSent

---

# Workflow 6

Payment Detection

Actor:

Blockchain Monitor

External System:

Solana RPC

Command:

DetectIncomingPayment

Event:

PaymentDetected

Policy:

StartConfirmationProcess

Event:

PaymentConfirmationStarted

Read Models:

IncomingPayments

---

Flow

Solana Transaction

PaymentDetected

PaymentConfirmationStarted

---

# Workflow 7

Payment Confirmation

Actor:

Blockchain Monitor

Command:

ConfirmPayment

Event:

PaymentConfirmed

Policy:

LinkPaymentToInvoice

Event:

InvoicePaid

Policy:

CreateLedgerEntries

Event:

LedgerEntriesCreated

Read Models:

PaymentHistory

RevenueDashboard

---

Flow

PaymentDetected

ConfirmPayment

PaymentConfirmed

InvoicePaid

LedgerEntriesCreated

---

# Workflow 8

Overpayment

Actor:

Blockchain Monitor

Command:

ProcessOverpayment

Event:

OverpaymentDetected

Policy:

FlagForReview

Event:

ManualReviewRequired

Read Models:

PaymentIssues

---

Flow

PaymentConfirmed

OverpaymentDetected

ManualReviewRequired

---

# Workflow 9

Duplicate Transaction

Actor:

Blockchain Monitor

Command:

ValidateTransaction

Event:

DuplicateTransactionDetected

Policy:

RejectProcessing

Event:

TransactionIgnored

Read Models:

SecurityEvents

---

Flow

Transaction Received

DuplicateTransactionDetected

TransactionIgnored

---

# Workflow 10

Settlement Creation

Actor:

System Scheduler

Command:

CreateSettlement

Event:

SettlementCreated

Policy:

CalculateFees

Event:

SettlementCalculated

Read Models:

SettlementQueue

---

Flow

CreateSettlement

SettlementCreated

SettlementCalculated

---

# Workflow 11

Settlement Execution

Actor:

Settlement Processor

External System:

Solana RPC

Command:

ExecuteSettlement

Event:

SettlementStarted

Event:

SettlementCompleted

Policy:

CreateLedgerEntries

Event:

SettlementLedgerRecorded

Read Models:

SettlementHistory

MerchantBalance

---

Flow

SettlementCreated

ExecuteSettlement

SettlementStarted

SettlementCompleted

SettlementLedgerRecorded

---

# Workflow 12

Settlement Failure

Actor:

Settlement Processor

Command:

HandleFailedSettlement

Event:

SettlementFailed

Policy:

ScheduleRetry

Event:

SettlementRetryScheduled

Read Models:

FailedSettlements

---

Flow

SettlementFailed

SettlementRetryScheduled

---

# Workflow 13

Webhook Registration

Actor:

Merchant

Command:

RegisterWebhook

Event:

WebhookRegistered

Policy:

GenerateWebhookSecret

Event:

WebhookActivated

Read Models:

WebhookList

---

Flow

RegisterWebhook

WebhookRegistered

WebhookActivated

---

# Workflow 14

Webhook Delivery

Actor:

Webhook Dispatcher

Command:

SendWebhook

Event:

WebhookDelivered

Policy:

StoreDeliveryLog

Event:

WebhookDeliveryRecorded

Read Models:

WebhookLogs

---

Flow

SendWebhook

WebhookDelivered

WebhookDeliveryRecorded

---

# Workflow 15

Webhook Failure

Actor:

Webhook Dispatcher

Command:

RetryWebhook

Event:

WebhookDeliveryFailed

Policy:

ScheduleRetry

Event:

WebhookRetryScheduled

Read Models:

WebhookFailures

---

Flow

WebhookDeliveryFailed

WebhookRetryScheduled

---

# Workflow 16

Merchant Suspension

Actor:

Admin

Command:

SuspendMerchant

Event:

MerchantSuspended

Policy:

DisableApiAccess

Policy:

PauseSettlements

Event:

MerchantAccessRevoked

Read Models:

MerchantStatus

---

Flow

SuspendMerchant

MerchantSuspended

MerchantAccessRevoked

---

# Workflow 17

Audit Trail

Actor:

System

Command:

RecordAuditEntry

Event:

AuditEntryCreated

Read Models:

AuditLog

---

Triggered By:

* MerchantCreated
* WalletCreated
* InvoiceCreated
* PaymentConfirmed
* SettlementCompleted
* WebhookDelivered

---

# Critical Event List

High Importance

MerchantRegistered

WalletCreated

InvoiceCreated

PaymentDetected

PaymentConfirmed

InvoicePaid

SettlementCreated

SettlementCompleted

WebhookDelivered

MerchantSuspended

AuditEntryCreated

---

# Event Ordering Rules

Must be preserved.

PaymentDetected

before

PaymentConfirmed

---

PaymentConfirmed

before

InvoicePaid

---

InvoicePaid

before

SettlementCreated

---

SettlementCreated

before

SettlementCompleted

---

SettlementCompleted

before

SettlementLedgerRecorded

---

# Future Event Streams

Phase 2

Multi-Chain

Events:

ChainAdded

ChainDisabled

---

Phase 3

Fraud Detection

Events:

FraudDetected

RiskScoreAssigned

AccountFlagged

---

Phase 4

AI Layer

Events:

InsightGenerated

ForecastGenerated

AnomalyDetected

RecommendationCreated

---

# Golden Rule

Commands express intent.

Events express facts.

Never confuse them.

Commands can fail.

Events already happened.

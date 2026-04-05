# Meridian Academy — System Design

## Overview

Meridian Academic Publishing & Fulfillment Portal is a full-stack, offline-first web application for academic institutions. It manages the complete lifecycle of academic content submissions, physical journal order fulfillment, peer reviews, and after-sales case management. The system runs entirely on a local network with no external service dependencies.

## Architecture

```
┌─────────────────────────────────────┐
│   Dioxus Frontend (WASM)            │
│   Role-aware SPA served via nginx   │
│   Port 8080                         │
└──────────────┬──────────────────────┘
               │ HTTP REST (localhost)
┌──────────────▼──────────────────────┐
│   Rocket Backend (Rust)             │
│   REST API with RBAC middleware     │
│   Port 8000                         │
└──────────────┬──────────────────────┘
               │ MySQL protocol
┌──────────────▼──────────────────────┐
│   MySQL 8                           │
│   All persistent data               │
│   Port 3306                         │
└─────────────────────────────────────┘
```

## Tech Stack

- **Frontend**: Dioxus (Rust compiled to WebAssembly via Trunk), served by nginx
- **Backend**: Rocket (Rust), REST API
- **Database**: MySQL 8
- **Language**: Rust for both frontend and backend in a single Cargo workspace

## Roles & RBAC

Access control is enforced at three levels: visible menu items, active buttons, and data scope (which records are returned).

| Role | Capabilities |
|------|-------------|
| Student | Own profile, own submissions, own orders, own reviews, own after-sales cases |
| Instructor | + Create/manage submissions, view student submissions in their courses |
| Academic Staff | + Content approval workflow, sensitive-word review queue |
| Administrator | Full access, user management, RBAC configuration, audit logs, payment config |

All permission changes are written to an immutable audit log: who changed what, when, and what the before/after state was.

## Authentication & Sessions

- Local-only username/password with salted hashing (bcrypt/argon2)
- 30-minute idle session timeout
- Password recovery: admin-issued one-time reset tokens (60-minute expiry), handed to users in person
- Account deletion: 30-day soft-delete hold before permanent removal
- "Export My Data": generates downloadable archive of the requester's own records only

## Database Schema (Key Tables)

| Table | Purpose |
|-------|---------|
| users | Accounts with hashed passwords, roles, soft-delete state |
| roles | Role definitions |
| permissions | Permission definitions |
| role_permissions | Role-to-permission mapping |
| user_roles | User-to-role assignment |
| orders | Subscription orders (monthly/quarterly/annual) |
| order_lines | Line items per order |
| fulfillment_events | Missing issues, reshipments, delays, discontinuations, edition changes |
| submissions | Multi-round submission records |
| submission_versions | Version history with timestamps |
| submission_files | File attachments with checksums |
| reviews | Initial reviews and follow-ups |
| review_images | Image attachments per review |
| after_sales_cases | Return/refund/exchange cases |
| after_sales_events | Status transition history |
| content_items | CMS content with metadata |
| content_versions | Version history for rollback |
| sensitive_words | Local dictionary for content governance |
| payment_transactions | Cash/Check/On-Account records |
| payment_holds | Escrow-style holds |
| reconciliation_reports | Nightly payment reconciliation |
| audit_logs | Immutable trail of all privileged actions |
| reset_tokens | One-time password reset tokens |
| addresses | US shipping addresses per user |
| notification_preferences | Per-user notification settings |

## Submission Flow

```
Draft → Submitted → Under Review → Revision Requested → Resubmitted → Accepted/Rejected
```
- Up to 10 resubmissions before deadline
- Every version saved with MM/DD/YYYY 12-hour timestamp
- Downloads watermarked with requester name + timestamp

## After-Sales Case Flow

```
Submitted → In Review → Awaiting Evidence → Arbitrated → Approved/Denied → Closed
```
- SLA: first response within 2 business days
- Resolution target: 7 business days

## Content Governance (on save)

1. Validate title (max 120 chars) and summary (max 500 chars)
2. Validate tags/keywords length
3. Auto-generate SEO fields (meta title, meta description, slug) deterministically
4. Run sensitive-word detection against local dictionary
5. If blocked → enter review queue for Academic Staff approval

## Payment Module

- Default: Cash/Check/On-Account (offline, no external gateway)
- Idempotent posting (duplicate submissions ignored)
- Escrow holds (reserve funds before finalizing)
- Refunds against prior transactions
- Nightly reconciliation reports
- Abnormal-order detection: flag high quantities or repeated refunds for manual review

## File Handling

- Allowed formats: PDF, DOCX, PNG, JPG
- Max size: 25 MB per file
- Safety: file signature (magic bytes) verification + SHA-256 checksum logging
- No cloud scanning

# Meridian Academy API Specification

Base URL: `http://localhost:8000`

## Authentication

All endpoints except `POST /api/auth/login`, `POST /api/auth/use-reset-token`, and `GET /health` require a Bearer token in the `Authorization` header.

Sessions expire after 30 minutes of idle time. The idle timer is refreshed on each authenticated request.

### Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Health check |
| POST | `/api/auth/login` | No | Login with username/password |
| POST | `/api/auth/provision` | Admin | Provision a new user account |
| GET | `/api/auth/me` | Yes | Get current user profile |
| POST | `/api/auth/change-password` | Yes | Change own password |
| POST | `/api/auth/generate-reset-token` | Admin | Generate one-time password reset token (60 min expiry) |
| POST | `/api/auth/use-reset-token` | No | Use reset token to set new password |
| POST | `/api/auth/request-deletion` | Yes | Request 30-day soft-delete hold |
| POST | `/api/auth/cancel-deletion` | Yes | Cancel pending deletion |
| GET | `/api/auth/export-my-data` | Yes | Download own data as JSON archive |
| POST | `/api/auth/logout` | Yes | Invalidate current session |

### POST /api/auth/login
```json
{ "username": "string", "password": "string" }
```
Response: `{ "token": "jwt", "user": { ... } }`

### POST /api/auth/provision
```json
{ "username": "string", "email": "string", "password": "string", "first_name": "string", "last_name": "string", "role": "student|instructor|academic_staff|administrator" }
```

### POST /api/auth/use-reset-token
```json
{ "token": "string", "new_password": "string" }
```

## Users

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/users` | Admin/Staff | List all users |
| GET | `/api/users/<id>` | Owner/Admin/Staff | Get user by ID |
| PUT | `/api/users/profile` | Yes | Update own profile |
| PUT | `/api/users/notification-prefs` | Yes | Update notification preferences |
| PUT | `/api/users/<id>/role` | Admin | Change user role |
| DELETE | `/api/users/<id>` | Admin | Deactivate user |
| GET | `/api/users/addresses` | Yes | List own addresses |
| POST | `/api/users/addresses` | Yes | Add shipping address |
| PUT | `/api/users/addresses/default` | Yes | Set default address |
| DELETE | `/api/users/addresses/<id>` | Yes | Delete address |
| GET | `/api/users/notifications` | Yes | Get notification inbox |
| PUT | `/api/users/notifications/<id>/read` | Yes | Mark notification read |

## Submissions

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/submissions` | Student/Instructor | Create submission (title <= 120 chars, summary <= 500 chars) |
| GET | `/api/submissions` | Yes | List submissions (scoped by role) |
| GET | `/api/submissions/<id>` | Owner/Admin/Staff | Get submission (IDOR enforced) |
| PUT | `/api/submissions/<id>` | Owner/Admin/Staff | Update submission |
| POST | `/api/submissions/<id>/versions` | Owner | Submit file version (max 10, PDF/DOCX/PNG/JPG, 25MB, magic-byte verified) |
| GET | `/api/submissions/<id>/versions` | Owner/Admin/Staff | List version history (timestamps in MM/DD/YYYY 12h format) |
| GET | `/api/submissions/<id>/versions/<n>/download` | Owner/Admin/Staff | Download with watermark |
| GET | `/api/submissions/my` | Yes | List own submissions |
| POST | `/api/submissions/<id>/approve` | Staff/Admin | Approve blocked submission |

### POST /api/submissions/<id>/versions
```json
{ "file_name": "paper.pdf", "file_data": "base64...", "form_data": "optional JSON" }
```

## Orders

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/orders` | Yes | Create order (monthly/quarterly/annual, multiple line items) |
| GET | `/api/orders` | Yes | List orders (scoped by role) |
| GET | `/api/orders/<id>` | Owner/Admin/Staff | Get order with line items |
| PUT | `/api/orders/<id>/status` | Admin/Staff | Update order status |
| GET | `/api/orders/my` | Yes | List own orders |
| POST | `/api/orders/split` | Admin/Staff | Split order by series |
| POST | `/api/orders/merge` | Admin/Staff | Merge orders from same user |
| POST | `/api/orders/fulfillment` | Admin/Staff | Log fulfillment event (reason required) |
| GET | `/api/orders/<id>/fulfillment` | Owner/Admin/Staff | List fulfillment events |
| GET | `/api/orders/<id>/reconciliation` | Owner/Admin/Staff | Get reconciliation records |
| PUT | `/api/orders/reconciliation/<id>` | Admin/Staff | Update reconciliation record |
| POST | `/api/orders/clear-flag` | Admin/Staff | Clear abnormal order flag |
| GET | `/api/orders/flagged` | Admin/Staff | List flagged orders |

### POST /api/orders
```json
{ "subscription_period": "monthly|quarterly|annual", "shipping_address_id": "optional", "line_items": [{ "publication_title": "string", "series_name": "optional", "quantity": 1, "unit_price": 29.99 }] }
```

## Reviews

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/reviews` | Yes | Create review (rating 1-5, title <= 120, order must be delivered) |
| POST | `/api/reviews/followup` | Yes | Follow-up review (1 per original, within 14 days) |
| GET | `/api/reviews` | Yes | List reviews (own for users, all for staff) |
| GET | `/api/reviews/<id>` | Owner/OrderOwner/Staff | Get review |
| POST | `/api/reviews/<id>/images` | Owner | Upload image (max 6, 5MB each) |
| GET | `/api/reviews/my` | Yes | List own reviews |

## After-Sales Cases

Status workflow: `submitted -> in_review -> awaiting_evidence -> arbitrated -> approved/denied -> closed`

SLA: First response within 2 business days (skips weekends), resolution target 7 business days.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/cases` | Yes | Create case (return/refund/exchange) |
| GET | `/api/cases` | Yes | List cases (scoped by role) |
| GET | `/api/cases/<id>` | Reporter/Staff | Get case with SLA timers |
| PUT | `/api/cases/<id>/status` | Staff/Admin | Transition case status |
| PUT | `/api/cases/<id>/assign` | Staff/Admin | Assign case to staff |
| POST | `/api/cases/<id>/comments` | Reporter/Assigned/Staff | Add comment (IDOR enforced) |
| GET | `/api/cases/<id>/comments` | Reporter/Staff | List comments |
| GET | `/api/cases/my` | Yes | List own cases |

## Payments

Methods: cash, check, on_account. Idempotent by `idempotency_key`. Third-party gateways disabled by default.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/payments` | Staff/Admin | Create payment (charge/hold/release/refund) |
| POST | `/api/payments/refund` | Staff/Admin | Refund against prior payment (idempotent) |
| GET | `/api/payments/order/<id>` | Owner/Staff | List payments for order |
| GET | `/api/payments/reconciliation-report` | Admin | On-demand reconciliation report (also auto-generated nightly by background scheduler) |
| GET | `/api/payments/abnormal-flags` | Staff/Admin | List abnormal order flags |
| POST | `/api/payments/abnormal-flags/<id>/clear` | Staff/Admin | Clear flag |

## Content Governance

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/content/sensitive-words` | Admin | List sensitive word dictionary |
| POST | `/api/content/sensitive-words` | Admin | Add word (replace/block) |
| DELETE | `/api/content/sensitive-words/<id>` | Admin | Remove word |
| POST | `/api/content/check` | Staff/Admin | Check text against dictionary |
| POST | `/api/content/items/<id>/submit` | Owner | Submit draft for review |
| POST | `/api/content/items/<id>/approve` | Staff/Admin | Approve content |
| POST | `/api/content/items/<id>/reject` | Staff/Admin | Reject content |
| POST | `/api/content/items/<id>/request-revision` | Staff/Admin | Request revision |
| POST | `/api/content/items/<id>/publish` | Staff/Admin | Publish accepted content |
| POST | `/api/content/items/<id>/rollback/<version>` | Owner/Staff/Admin | Rollback to previous version |

## Admin

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/admin/dashboard` | Admin | Dashboard statistics |
| GET | `/api/admin/audit-log` | Admin | Immutable audit log (last 200 entries, returns `{ "logs": [...] }`) |
| GET | `/api/admin/audit-logs` | Admin | Same as above, returns flat array (used by frontend) |
| GET | `/api/admin/settings` | Admin | System settings and constants |
| POST | `/api/admin/cleanup-soft-deleted` | Admin | Permanently delete expired soft-deleted users |

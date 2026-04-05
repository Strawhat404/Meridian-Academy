# Meridian Academy — REST API Specification

All endpoints are served by the Rocket backend at `http://localhost:8000/api`.

Authentication uses a session token passed as a Bearer token in the `Authorization` header.

---

## Auth

### POST /api/auth/login
Login with username and password.

Request: `{ "username": string, "password": string }`
Response: `{ "token": string, "user": { "id", "username", "role" } }`
Errors: 401 (invalid credentials), 423 (account locked)

### POST /api/auth/logout
Invalidate the current session token.

Response: `{ "success": true }`

### POST /api/auth/reset-password
Use a one-time reset token to set a new password.

Request: `{ "token": string, "new_password": string }`
Response: `{ "success": true }`
Errors: 400 (token expired or invalid)

### GET /api/auth/me
Get the current authenticated user.

Response: `{ "id", "username", "role", "email" }`

---

## Users

### GET /api/users
Admin only. List all users with pagination.

Query: `?page=1&per_page=20&role=student`

### GET /api/users/:id
Get user profile. Users can only access their own; admins can access any.

### PUT /api/users/:id
Update profile fields (name, email).

### DELETE /api/users/:id
Soft-delete user account (30-day hold).

### GET /api/users/:id/export
Export all of the user's own data as a downloadable JSON archive.

### GET /api/users/:id/addresses
List shipping addresses for a user.

### POST /api/users/:id/addresses
Add a new shipping address.

### PUT /api/users/:id/addresses/:addr_id
Update an address. Set `"is_default": true` to make it the default.

### DELETE /api/users/:id/addresses/:addr_id
Remove an address.

---

## Submissions

### GET /api/submissions
List submissions. Students see own; instructors see their courses'; admins see all.

### POST /api/submissions
Create a new submission draft.

Request: `{ "title": string, "template_id": string, "deadline": string }`

### GET /api/submissions/:id
Get submission with full version history.

### POST /api/submissions/:id/versions
Submit a new version (resubmit). Max 10 versions before deadline.

Request: multipart form with fields + file attachments.

### GET /api/submissions/:id/versions/:version/download
Download a specific version. Response includes watermark metadata in headers.

---

## Orders

### GET /api/orders
List orders for the current user (or all for admin).

### POST /api/orders
Create a new subscription order.

Request: `{ "period": "monthly|quarterly|annual", "lines": [{ "series_id", "quantity" }] }`

### GET /api/orders/:id
Get order details with line items and fulfillment events.

### POST /api/orders/:id/split
Split an order containing multiple series into separate orders.

### POST /api/orders/merge
Merge multiple orders from the same subscriber.

Request: `{ "order_ids": [string] }`

### GET /api/orders/:id/reconciliation
Get reconciliation view: expected vs. actual receipt per issue.

### POST /api/orders/:id/fulfillment-events
Log a fulfillment event.

Request: `{ "event_type": "missing|reshipment|delay|discontinuation|edition_change", "issue_id": string, "reason": string }`

---

## Reviews

### GET /api/reviews
List reviews. Users see own; admins see all.

### POST /api/reviews
Post an initial review.

Request: multipart with `{ "order_id", "rating", "body" }` + up to 6 images (5 MB each).

### POST /api/reviews/:id/followup
Post a follow-up (within 14 days of original review, one per review).

### GET /api/reviews/:id
Get review with follow-up and images.

---

## After-Sales Cases

### GET /api/cases
List after-sales cases for current user (or all for admin/staff).

### POST /api/cases
Open a new case.

Request: `{ "order_id": string, "case_type": "return|refund|exchange", "description": string }`

### GET /api/cases/:id
Get case details with status history and SLA timers.

### PUT /api/cases/:id/status
Update case status (staff/admin only).

Request: `{ "status": "in_review|awaiting_evidence|arbitrated|approved|denied|closed", "note": string }`

---

## Content

### GET /api/content
List content items.

### POST /api/content
Create a content draft. Triggers metadata governance on save.

Request: `{ "title": string, "summary": string, "body": string, "tags": [string] }`

### PUT /api/content/:id
Update content. Re-runs governance validation.

### POST /api/content/:id/submit
Submit for review (Academic Staff approval required if flagged).

### POST /api/content/:id/approve
Academic Staff or Admin approves and publishes content.

### POST /api/content/:id/reject
Reject content with a reason.

### GET /api/content/:id/versions
Get version history.

### POST /api/content/:id/rollback/:version
Roll back to a previous version.

---

## Payments

### GET /api/payments
List payment transactions (admin only).

### POST /api/payments
Record a new transaction.

Request: `{ "order_id": string, "method": "cash|check|on_account", "amount_cents": number, "idempotency_key": string }`

### POST /api/payments/:id/refund
Issue a refund against a prior transaction.

### GET /api/payments/reconciliation
Get nightly reconciliation report.

---

## Admin

### GET /api/admin/users
List all users with roles and status.

### POST /api/admin/users/:id/reset-token
Generate a one-time password reset token (60-minute expiry).

### PUT /api/admin/users/:id/role
Change a user's role. Logged to audit trail.

### GET /api/admin/audit-logs
Get immutable audit log entries with pagination.

### GET /api/admin/sensitive-words
List sensitive words in the local dictionary.

### POST /api/admin/sensitive-words
Add a sensitive word with policy (replace or block).

### DELETE /api/admin/sensitive-words/:id
Remove a sensitive word.

### GET /api/admin/flagged-orders
List orders flagged for manual review (abnormal patterns).

### POST /api/admin/flagged-orders/:id/clear
Clear a flag and allow the order to proceed.

---

## Health

### GET /health
Returns 200 OK. Used by Docker healthcheck.

# Fresh Fix Verification Rerun (Static-Only)
Timestamp (UTC): 2026-04-10T15:42:54Z
Method: Re-verified from current files only. No runtime execution, no Docker, no tests run.

## Summary
- Fixed: 12
- Partially Fixed: 0
- Not Fixed: 0

## Per-Issue Results

1) Core domain aligned but missing full role-oriented web operations + RBAC/session gaps
- Status: **Fixed**
- Evidence:
  - Staff/admin management route and guard: `repo/frontend/src/main.rs:36-37`, `repo/frontend/src/main.rs:340-349`, `repo/frontend/src/main.rs:442-447`
  - Admin cases tab and operations surfaced in UI: `repo/frontend/src/pages/admin.rs:136-143`, `repo/frontend/src/pages/admin.rs:169`, `repo/frontend/src/pages/admin.rs:405-427`
  - Staff/admin case detail workflow (status/assign/comments): `repo/frontend/src/pages/cases.rs:207-223`, `repo/frontend/src/pages/cases.rs:280-303`, `repo/frontend/src/pages/cases.rs:317-329`, `repo/frontend/src/pages/cases.rs:372-381`
  - RBAC/session hardening in guard: `repo/backend/src/middleware/auth_guard.rs:104-110`, `repo/backend/src/middleware/auth_guard.rs:112-125`, `repo/backend/src/middleware/auth_guard.rs:148-156`

2) Authz tied to JWT claim role; no fresh DB role/state per request
- Status: **Fixed**
- Evidence:
  - Fresh role/active/deleted DB query: `repo/backend/src/middleware/auth_guard.rs:104-110`
  - Deactivated/soft-deleted denial in guard: `repo/backend/src/middleware/auth_guard.rs:114-125`
  - Permissions resolved from DB role: `repo/backend/src/middleware/auth_guard.rs:148-156`

3) Error-handling detail: default-address success without effective change + fragile PDF watermark internals
- Status: **Fixed**
- Evidence (default address state checks):
  - Ownership/existence check before clearing defaults: `repo/backend/src/routes/users.rs:197-207`
  - Affected-row check before returning OK: `repo/backend/src/routes/users.rs:214-220`
- Evidence (PDF robustness):
  - Watermark now uses `lopdf` with parser/object/xref management (explicitly replacing hand-rolled parsing): `repo/backend/src/routes/submissions.rs:52-56`, `repo/backend/src/routes/submissions.rs:57-65`, `repo/backend/src/routes/submissions.rs:147-154`

4) Offline/local-first alignment strong, but role-operational UX and robust RBAC semantics gaps
- Status: **Fixed**
- Evidence:
  - Role-operational UI present: `repo/frontend/src/pages/admin.rs:136-143`, `repo/frontend/src/pages/cases.rs:207-223`
  - Robust RBAC semantics (fresh DB checks): `repo/backend/src/middleware/auth_guard.rs:104-110`, `repo/backend/src/middleware/auth_guard.rs:148-156`

5) RBAC permissions stale after role change (token claim role used)
- Status: **Fixed**
- Evidence:
  - Permission resolution uses DB role: `repo/backend/src/middleware/auth_guard.rs:148-156`
  - Session invalidation on role change: `repo/backend/src/routes/users.rs:113-115`

6) Deactivated/soft-deleted account state not rechecked during authenticated requests
- Status: **Fixed**
- Evidence:
  - `repo/backend/src/middleware/auth_guard.rs:114-125`

7) Staff/admin after-sales management backend-capable but not surfaced in frontend workflow
- Status: **Fixed**
- Evidence:
  - Admin case management tab + list: `repo/frontend/src/pages/admin.rs:136-143`, `repo/frontend/src/pages/admin.rs:169`
  - Status operations in UI: `repo/frontend/src/pages/admin.rs:405-427`
  - Dedicated staff/admin detail page: `repo/frontend/src/main.rs:36-37`, `repo/frontend/src/main.rs:442-447`, `repo/frontend/src/pages/cases.rs:207-223`

8) Address default invariant can silently end with zero defaults
- Status: **Fixed**
- Evidence:
  - Prevents clearing for nonexistent/non-owned target: `repo/backend/src/routes/users.rs:197-207`
  - Then clear/set only valid target: `repo/backend/src/routes/users.rs:209-215`

9) Requested default set without affected-row check; always returns OK
- Status: **Fixed**
- Evidence:
  - `repo/backend/src/routes/users.rs:217-220`

10) Test suite gaps around account lifecycle/security-sensitive admin flows
- Status: **Fixed**
- Evidence:
  - Staff cannot deactivate, admin can: `repo/API_tests/src/lib.rs:1657-1678`
  - Deactivated token rejected: `repo/API_tests/src/lib.rs:1684-1710`
  - Role-change invalidates old session: `repo/API_tests/src/lib.rs:1716-1737`
  - Reset-token admin boundary/lifecycle: `repo/API_tests/src/lib.rs:1811-1815`, `repo/API_tests/src/lib.rs:1859-1868`, `repo/API_tests/src/lib.rs:1879-1883`
  - Export auth boundaries: `repo/API_tests/src/lib.rs:1894-1919`
  - Admin-only cleanup: `repo/API_tests/src/lib.rs:1932-1941`
  - Creation endpoints auth rejection: `repo/API_tests/src/lib.rs:1949-1973`
  - Deactivated users denied on creation endpoints: `repo/API_tests/src/lib.rs:1991-2013`, `repo/API_tests/src/lib.rs:2017-2037`, `repo/API_tests/src/lib.rs:2041-2061`

11) Route-level authorization gap for creation endpoints (orders/reviews/cases)
- Status: **Fixed**
- Evidence:
  - Orders permission check: `repo/backend/src/routes/orders.rs:19-20`
  - Reviews permission check: `repo/backend/src/routes/reviews.rs:12-13`
  - Cases permission check: `repo/backend/src/routes/cases.rs:58-59`

12) Function-level authorization used token claim role instead of fresh DB role
- Status: **Fixed**
- Evidence:
  - DB role drives permission lookup and assigned auth user role: `repo/backend/src/middleware/auth_guard.rs:148-156`, `repo/backend/src/middleware/auth_guard.rs:160-165`

## Boundary Notes
- Static-only conclusions.
- No runtime validation claimed.
- No project startup, Docker, or test execution performed.

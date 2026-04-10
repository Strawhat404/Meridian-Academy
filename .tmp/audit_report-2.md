# Meridian Academy Static Delivery Acceptance & Architecture Audit

Date: 2026-04-10  
Scope: Static-only repository audit (no runtime execution)

## 1. Verdict
- Overall conclusion: **Partial Pass**
- Rationale: The repository is substantial and implements many core flows, but has material security and requirement-fit gaps (notably RBAC/session invalidation behavior and incomplete role-management UI coverage) that prevent a full pass.

## 2. Scope and Static Verification Boundary
- What was reviewed:
  - Top-level and repo-level documentation/config: `README.md`, `repo/README.md`, `docs/design.md`, `docs/api-spec.md`, `repo/.env.example`, `repo/docker-compose.yml`
  - Backend entrypoints, middleware, routes, models, SQL migrations
  - Frontend route map, page components, services, stylesheet
  - Unit and API/integration test code and test manifests
- What was not reviewed:
  - Runtime behavior under real browser/backend/database execution
  - External network behavior, container orchestration behavior, production deployment behavior
- What was intentionally not executed:
  - Project startup, tests, Docker, database, external services
- Claims requiring manual verification:
  - Actual validity/renderability of generated watermarked files (especially arbitrary PDFs)
  - Full UX behavior across browsers/devices
  - Runtime race/consistency under concurrent writes

## 3. Repository / Requirement Mapping Summary
- Prompt core goal mapped: offline-capable full-stack portal for academic submissions, orders/fulfillment, reviews, after-sales, RBAC, auditability, local auth/session, content governance, and offline payments.
- Main implementation areas mapped:
  - Backend Rocket API modules under `repo/backend/src/routes/*.rs`
  - MySQL schema under `repo/backend/src/migrations/001_initial.sql`
  - Dioxus frontend routes/pages under `repo/frontend/src/main.rs` and `repo/frontend/src/pages/*.rs`
  - Static tests under `repo/unit_tests/src/lib.rs` and `repo/API_tests/src/lib.rs`

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- Conclusion: **Pass**
- Rationale: Startup, configuration, and test docs are present; project structure and entrypoints are statically discoverable and consistent.
- Evidence:
  - `README.md:8`
  - `repo/README.md:3`
  - `repo/README.md:23`
  - `repo/Cargo.toml:1`
  - `repo/backend/src/main.rs:70`
  - `repo/frontend/src/main.rs:12`

#### 1.2 Material deviation from Prompt
- Conclusion: **Partial Pass**
- Rationale: Core domain is aligned, but delivery misses full role-oriented operations in the web experience (notably case-management operations for staff/admin are API-only, not exposed as a management UI), and RBAC/session behavior has material security-fit gaps.
- Evidence:
  - Staff/admin case mutation endpoints exist backend: `repo/backend/src/routes/cases.rs:145`, `repo/backend/src/routes/cases.rs:180`
  - Frontend cases page only lists own cases and new-case form: `repo/frontend/src/pages/cases.rs:37`, `repo/frontend/src/pages/cases.rs:95`
  - Session permissions derived from token role claim (not fresh DB role): `repo/backend/src/middleware/auth_guard.rs:111`

### 2. Delivery Completeness

#### 2.1 Core explicit requirement coverage
- Conclusion: **Partial Pass**
- Rationale: Many explicit requirements are implemented (submission limits/deadline, watermark download, split/merge, reconciliation, follow-up window, image limits, reset token expiry, soft delete, export-my-data, sensitive words, offline payment methods), but some requirements are incompletely represented in the end-user web experience and notification behavior is mostly passive.
- Evidence:
  - Submission versions/deadline/file checks: `repo/backend/src/routes/submissions.rs:522`, `repo/backend/src/routes/submissions.rs:527`, `repo/backend/src/routes/submissions.rs:543`
  - Watermarked download path: `repo/backend/src/routes/submissions.rs:615`
  - Split/merge/reconciliation: `repo/backend/src/routes/orders.rs:219`, `repo/backend/src/routes/orders.rs:290`, `repo/backend/src/routes/orders.rs:443`
  - Follow-up + 14-day window + max images: `repo/backend/src/routes/reviews.rs:63`, `repo/backend/src/routes/reviews.rs:81`, `repo/backend/src/routes/reviews.rs:181`
  - Reset token + soft delete + export: `repo/backend/src/routes/auth.rs:260`, `repo/backend/src/routes/auth.rs:333`, `repo/backend/src/routes/auth.rs:362`
  - Notification inbox endpoints are read/update only (no producer logic found): `repo/backend/src/routes/users.rs:209`, `repo/backend/src/routes/users.rs:226`

#### 2.2 End-to-end 0→1 deliverable vs demo/fragment
- Conclusion: **Pass**
- Rationale: Structured multi-module workspace with backend/frontend/tests/docs; not a single-file demo.
- Evidence:
  - Workspace modules: `repo/Cargo.toml:2`
  - Backend routes breadth: `repo/backend/src/routes/mod.rs:16`
  - Frontend route/page breadth: `repo/frontend/src/main.rs:20`
  - Tests present: `repo/unit_tests/src/lib.rs:1`, `repo/API_tests/src/lib.rs:1`

### 3. Engineering and Architecture Quality

#### 3.1 Structure and module decomposition
- Conclusion: **Pass**
- Rationale: Clear separation of routes/models/middleware/frontend pages/services and SQL migrations.
- Evidence:
  - Backend module split: `repo/backend/src/routes/mod.rs:1`, `repo/backend/src/models/mod.rs:1`, `repo/backend/src/middleware/auth_guard.rs:1`
  - Frontend split: `repo/frontend/src/main.rs:1`, `repo/frontend/src/pages/mod.rs:1`, `repo/frontend/src/services/mod.rs:1`

#### 3.2 Maintainability/extensibility
- Conclusion: **Partial Pass**
- Rationale: Overall structure is maintainable, but key authz behavior is tightly coupled to JWT claims role and does not re-evaluate user state/role from DB per request, weakening correctness after role/deactivation changes.
- Evidence:
  - Role-permission resolution uses claim role: `repo/backend/src/middleware/auth_guard.rs:106`, `repo/backend/src/middleware/auth_guard.rs:111`
  - Role changes happen in DB: `repo/backend/src/routes/users.rs:110`
  - Deactivation changes DB flag only: `repo/backend/src/routes/users.rs:129`

### 4. Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API shape
- Conclusion: **Partial Pass**
- Rationale: Error handling/logging exists broadly; key validation rules are implemented. However, some endpoints return success without asserting effective state change (e.g., default-address update), and file watermark logic uses fragile PDF internals.
- Evidence:
  - Logging/error mapping examples: `repo/backend/src/routes/auth.rs:83`, `repo/backend/src/routes/orders.rs:92`, `repo/backend/src/routes/reviews.rs:130`
  - Address default update without affected-row check: `repo/backend/src/routes/users.rs:194`, `repo/backend/src/routes/users.rs:197`
  - PDF watermark uses approximate xref offsets: `repo/backend/src/routes/submissions.rs:101`, `repo/backend/src/routes/submissions.rs:102`, `repo/backend/src/routes/submissions.rs:105`

#### 4.2 Product-level organization vs sample
- Conclusion: **Pass**
- Rationale: The codebase shape and breadth look like a real product baseline, not just tutorial scaffolding.
- Evidence:
  - Domain breadth in schema: `repo/backend/src/migrations/001_initial.sql:92`, `repo/backend/src/migrations/001_initial.sql:131`, `repo/backend/src/migrations/001_initial.sql:228`, `repo/backend/src/migrations/001_initial.sql:289`

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business goal/constraints fit
- Conclusion: **Partial Pass**
- Rationale: Strong alignment to offline/local-first architecture and most domain flows. Material gaps remain in full role-operational web UX and robust RBAC semantics after role/account-state changes.
- Evidence:
  - Offline/local architecture: `docs/design.md:5`, `repo/frontend/src/services/api.rs:5`
  - Role dashboard/nav: `repo/frontend/src/main.rs:262`, `repo/frontend/src/components/nav.rs:16`
  - RBAC stale-role risk: `repo/backend/src/middleware/auth_guard.rs:111`
  - Case management UI limitation (reporter-centric): `repo/frontend/src/pages/cases.rs:39`, `repo/frontend/src/pages/cases.rs:107`

### 6. Aesthetics (frontend)

#### 6.1 Visual and interaction quality
- Conclusion: **Pass**
- Rationale: UI has clear hierarchy, consistent styling, responsive layout, distinguishable sections, and basic interaction states (hover, badges, buttons). No major static rendering contradictions found.
- Evidence:
  - Layout/nav separation: `repo/frontend/src/components/layout.rs:7`, `repo/frontend/src/components/nav.rs:11`
  - Consistent style system and responsive rules: `repo/frontend/assets/style.css:2`, `repo/frontend/assets/style.css:188`, `repo/frontend/assets/style.css:285`

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High

1) **High** - RBAC permissions can remain stale after role change (token claim role used for permission resolution)
- Conclusion: **Fail**
- Evidence:
  - Permission lookup bound to JWT claim role: `repo/backend/src/middleware/auth_guard.rs:106`, `repo/backend/src/middleware/auth_guard.rs:111`
  - Role is mutable in DB: `repo/backend/src/routes/users.rs:110`
- Impact:
  - Users can retain outdated effective permissions until token/session replacement, undermining auditable permission-change enforcement.
- Minimum actionable fix:
  - In auth guard, resolve current role (or direct effective permissions) from `users`/`role_permissions` by `claims.sub` each request; do not trust role in token for authorization decisions.

2) **High** - Deactivated/soft-deleted account state is not rechecked during authenticated requests
- Conclusion: **Fail**
- Evidence:
  - Auth guard validates session record only: `repo/backend/src/middleware/auth_guard.rs:54`
  - Deactivation toggles `users.is_active = false`: `repo/backend/src/routes/users.rs:129`
  - Login gate enforces active/non-soft-deleted only at login time: `repo/backend/src/routes/auth.rs:78`
- Impact:
  - Existing valid sessions can continue operating after account deactivation/soft-delete request until session expiry/logout.
- Minimum actionable fix:
  - During request guard, join/read `users` and require `is_active=true` and `soft_deleted_at IS NULL`; invalidate session otherwise.

3) **High** - Staff/admin after-sales management is backend-capable but not materially surfaced in frontend workflow
- Conclusion: **Partial Fail**
- Evidence:
  - Backend supports status/assignment operations: `repo/backend/src/routes/cases.rs:145`, `repo/backend/src/routes/cases.rs:180`
  - Frontend cases page only uses `GET /api/cases/my` and create case form: `repo/frontend/src/pages/cases.rs:39`, `repo/frontend/src/pages/cases.rs:95`
- Impact:
  - “Single web experience” for staff/admin case operations is incomplete; core management actions require out-of-band API tooling.
- Minimum actionable fix:
  - Add privileged case-management UI (list all cases, status transition controls, assignment controls, threaded comments with scope checks).

4) **High (Suspected Risk)** - PDF watermark implementation uses heuristic xref offsets likely fragile across real PDFs
- Conclusion: **Cannot Confirm Statistically (suspected defect)**
- Evidence:
  - Approximate xref/object offsets hardcoded: `repo/backend/src/routes/submissions.rs:101`, `repo/backend/src/routes/submissions.rs:102`, `repo/backend/src/routes/submissions.rs:105`
  - Assumes `/Parent 1 0 R` page tree linkage: `repo/backend/src/routes/submissions.rs:90`
- Impact:
  - Risk of corrupted/unopenable watermarked PDFs or invalid structure on non-trivial source files.
- Minimum actionable fix:
  - Use a proper PDF library to append watermark content and regenerate cross-reference/trailer correctly.

### Medium

5) **Medium** - Address default invariant can silently end with zero defaults
- Conclusion: **Partial Fail**
- Evidence:
  - Clears all defaults unconditionally: `repo/backend/src/routes/users.rs:190`
  - Sets requested default without checking affected rows; always returns OK: `repo/backend/src/routes/users.rs:194`, `repo/backend/src/routes/users.rs:197`
- Impact:
  - “Single default address” rule can be violated if invalid address id is submitted.
- Minimum actionable fix:
  - Validate target address ownership/existence first; enforce exactly one default in a transaction and verify row counts.

6) **Medium** - Notification preference/inbox exists but no backend producer flow found for in-app banner events
- Conclusion: **Partial Pass**
- Evidence:
  - Notification read/list endpoints only: `repo/backend/src/routes/users.rs:209`, `repo/backend/src/routes/users.rs:226`
  - No notification insert logic found in backend routes/services scan
- Impact:
  - Users can configure/see inbox, but actionable in-app banner generation appears incomplete.
- Minimum actionable fix:
  - Add event-driven notification creation on key domain actions respecting user preference flags.

7) **Medium** - Test suite has notable gaps around account lifecycle/security-sensitive admin flows
- Conclusion: **Insufficient coverage**
- Evidence:
  - Routes exist for reset/export/deletion cleanup: `repo/backend/src/routes/auth.rs:260`, `repo/backend/src/routes/auth.rs:289`, `repo/backend/src/routes/auth.rs:333`, `repo/backend/src/routes/auth.rs:362`, `repo/backend/src/routes/admin.rs:103`
  - No corresponding integration test functions in test index for these flows: `repo/API_tests/src/lib.rs:86`, `repo/API_tests/src/lib.rs:1680`
- Impact:
  - Regressions in sensitive account recovery/deletion/export flows may go undetected.
- Minimum actionable fix:
  - Add API tests for reset-token lifecycle, export scoping, deletion hold behavior, and cleanup authorization.

### Low

8) **Low** - `.env` with default secrets/credentials is committed in repo tree
- Conclusion: **Risk acknowledged**
- Evidence:
  - `repo/.env:1`
- Impact:
  - Encourages insecure defaults if reused outside local/offline context.
- Minimum actionable fix:
  - Keep `.env` untracked in VCS; rely on `.env.example` and environment provisioning per deployment profile.

## 6. Security Review Summary

- Authentication entry points: **Pass**
  - Evidence: Login/provision/reset/logout endpoints and JWT/session guard exist: `repo/backend/src/routes/auth.rs:75`, `repo/backend/src/routes/auth.rs:131`, `repo/backend/src/middleware/auth_guard.rs:19`
  - Note: Passwords are hashed with bcrypt and sessions are tracked.

- Route-level authorization: **Partial Pass**
  - Evidence: Many privileged endpoints enforce permission/privileged checks: `repo/backend/src/routes/admin.rs:11`, `repo/backend/src/routes/orders.rs:220`, `repo/backend/src/routes/payments.rs:11`
  - Gap: Some creation endpoints rely only on authenticated ownership semantics (no explicit permission check): `repo/backend/src/routes/orders.rs:19`, `repo/backend/src/routes/reviews.rs:12`, `repo/backend/src/routes/cases.rs:58`

- Object-level authorization: **Pass**
  - Evidence: IDOR checks are broadly implemented in submissions/orders/reviews/cases/comments: `repo/backend/src/routes/submissions.rs:447`, `repo/backend/src/routes/orders.rs:170`, `repo/backend/src/routes/reviews.rs:152`, `repo/backend/src/routes/cases.rs:199`

- Function-level authorization: **Partial Pass**
  - Evidence: Fine-grained permission checks exist for many admin/content/payment functions: `repo/backend/src/routes/content.rs:14`, `repo/backend/src/routes/payments.rs:212`
  - Gap: Permission evaluation depends on token claim role instead of fresh DB role: `repo/backend/src/middleware/auth_guard.rs:111`

- Tenant/user data isolation: **Pass**
  - Evidence: User-scoped queries and ownership checks across key resource endpoints: `repo/backend/src/routes/users.rs:144`, `repo/backend/src/routes/orders.rs:147`, `repo/backend/src/routes/submissions.rs:417`

- Admin/internal/debug endpoint protection: **Partial Pass**
  - Evidence: Admin endpoints are permission-protected: `repo/backend/src/routes/admin.rs:11`, `repo/backend/src/routes/admin.rs:41`
  - Gap: `cleanup_soft_deleted` uses `admin.dashboard` permission instead of a narrower dedicated cleanup/admin-only permission: `repo/backend/src/routes/admin.rs:105`

## 7. Tests and Logging Review

- Unit tests: **Partial Pass**
  - Evidence: Many unit tests cover constants/validators/helpers: `repo/unit_tests/src/lib.rs:27`, `repo/unit_tests/src/lib.rs:96`, `repo/unit_tests/src/lib.rs:183`
  - Gap: Several tests are tautological or not exercising production route logic (e.g., simple boolean assertions): `repo/unit_tests/src/lib.rs:358`, `repo/unit_tests/src/lib.rs:411`

- API/integration tests: **Partial Pass**
  - Evidence: Broad coverage for auth basics, RBAC, IDOR, fulfillment/reconciliation, follow-up/image limits, logout invalidation: `repo/API_tests/src/lib.rs:98`, `repo/API_tests/src/lib.rs:156`, `repo/API_tests/src/lib.rs:247`, `repo/API_tests/src/lib.rs:667`, `repo/API_tests/src/lib.rs:1176`
  - Gap: Missing coverage for reset-token flow, export-my-data scope, account deletion hold/cleanup, and role-change/deactivation effects on active sessions.

- Logging categories/observability: **Pass**
  - Evidence: Structured error/warn/info/debug logs across critical paths: `repo/backend/src/routes/auth.rs:83`, `repo/backend/src/routes/submissions.rs:344`, `repo/backend/src/main.rs:106`, `repo/backend/src/routes/payments.rs:248`

- Sensitive-data leakage risk in logs/responses: **Partial Pass**
  - Evidence: Password values are not directly logged; failure details include contextual IDs/actions.
  - Residual risk: Some audit details and logs include user identifiers and action metadata; acceptable for audit use but requires operational log access control.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist: `repo/unit_tests/src/lib.rs:1`
- API/integration tests exist: `repo/API_tests/src/lib.rs:1`
- Test frameworks: Rust `#[test]` and `#[tokio::test]`
- Test entry points documented:
  - `repo/README.md:23`
  - `repo/run_tests.sh:52`
  - `repo/run_tests.sh:63`

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Auth 401 on missing/invalid credentials | `repo/API_tests/src/lib.rs:99`, `repo/API_tests/src/lib.rs:108` | `assert_eq!(resp.status(), 401)` | sufficient | Low | Keep regression tests |
| Admin-only provisioning | `repo/API_tests/src/lib.rs:128`, `repo/API_tests/src/lib.rs:140` | 401/403 enforcement checks | sufficient | Low | Add negative test for forged role claim |
| IDOR on submissions/orders/cases/comments | `repo/API_tests/src/lib.rs:248`, `repo/API_tests/src/lib.rs:282`, `repo/API_tests/src/lib.rs:327`, `repo/API_tests/src/lib.rs:489` | 403/404 on cross-user access | sufficient | Moderate (not all endpoints) | Add IDOR tests for payments/order list and content item lifecycle |
| Submission validation (title/summary) | `repo/API_tests/src/lib.rs:219`, `repo/API_tests/src/lib.rs:569` | `422` for over-limit | basically covered | No test for deadline expiry and 11th version edge via API | Add tests for post-deadline version submit and version limit reached |
| Watermarked download contract | `repo/API_tests/src/lib.rs:758`, `repo/API_tests/src/lib.rs:842` | content-type and watermark headers | basically covered | Does not prove arbitrary PDF validity | Add corpus-based PDF roundtrip/openability verification |
| Review follow-up constraints + image limits | `repo/API_tests/src/lib.rs:974`, `repo/API_tests/src/lib.rs:1031`, `repo/API_tests/src/lib.rs:1124` | 409 on second follow-up, 422 on 7th image | sufficient | No explicit expired-14-day follow-up case | Add test with synthetic old timestamp fixture |
| Fulfillment/reconciliation core flow | `repo/API_tests/src/lib.rs:1237`, `repo/API_tests/src/lib.rs:1294`, `repo/API_tests/src/lib.rs:1363`, `repo/API_tests/src/lib.rs:1550` | expected/received/status transitions | sufficient | None major | Keep |
| Session invalidation on logout | `repo/API_tests/src/lib.rs:1176` | token rejected post logout | sufficient | No tests for deactivation/role-change session effect | Add tests for active-session behavior after role update/deactivate |
| Password reset token flow | none found | n/a | missing | Security-critical account recovery path untested | Add end-to-end generate/use/expiry/reuse tests |
| Export My Data scoping | none found | n/a | missing | Data isolation export path untested | Add test that export excludes other users’ records |
| Soft-delete hold and cleanup | none found | n/a | missing | Lifecycle and retention behavior untested | Add tests for request/cancel/cleanup authorization & timing |

### 8.3 Security Coverage Audit
- Authentication: **Basically covered** (login invalid, missing auth, logout invalidation)  
  Evidence: `repo/API_tests/src/lib.rs:99`, `repo/API_tests/src/lib.rs:108`, `repo/API_tests/src/lib.rs:1176`
- Route authorization: **Basically covered** (many 403 tests), but not comprehensive for all privileged endpoints  
  Evidence: `repo/API_tests/src/lib.rs:157`, `repo/API_tests/src/lib.rs:168`, `repo/API_tests/src/lib.rs:179`
- Object-level authorization: **Covered for key resources**  
  Evidence: `repo/API_tests/src/lib.rs:248`, `repo/API_tests/src/lib.rs:282`, `repo/API_tests/src/lib.rs:489`
- Tenant/data isolation: **Basically covered** for common read/write paths  
  Evidence: `repo/API_tests/src/lib.rs:248`, `repo/API_tests/src/lib.rs:387`
- Admin/internal protection: **Partially covered** (dashboard/users/payment/content checks exist), but cleanup/reset/export lifecycle controls are not covered

### 8.4 Final Coverage Judgment
- **Partial Pass**
- Boundary:
  - Major auth/RBAC/IDOR/reconciliation happy-path and key denial paths are covered.
  - Uncovered security-critical flows (reset/export/deletion lifecycle and role/deactivation-with-active-session effects) mean severe defects could still remain undetected while tests pass.

## 9. Final Notes
- Audit conclusions are static and evidence-based; runtime-dependent claims were not asserted as proven.
- Most functionality is present, but high-priority fixes are needed in authorization/session-state correctness and role-complete frontend management flows before acceptance as fully compliant.

1. Verdict
- Overall conclusion: Partial Pass

2. Scope and Static Verification Boundary
- Reviewed: repository structure, docs, backend routes/models/migrations/middleware, frontend pages/services/routes, test code/manifests/scripts.
- Primary evidence sources: `README.md`, `repo/README.md`, `docs/api-spec.md`, `repo/docs/api-spec.md`, `repo/backend/src/**`, `repo/frontend/src/**`, `repo/API_tests/src/lib.rs`, `repo/unit_tests/src/lib.rs`.
- Not reviewed: runtime behavior, browser rendering, DB execution, network behavior, Docker/container behavior, real session timeout timing.
- Intentionally not executed: project startup, tests, Docker, external services.
- Manual verification required for: real offline/LAN behavior, true watermark rendering in binary formats, scheduler timing at midnight, and end-to-end UX flows.

3. Repository / Requirement Mapping Summary
- Prompt core goal: offline-runnable academic publishing/fulfillment portal with role-aware flows across user center, submissions, orders/fulfillment/reconciliation, reviews, after-sales, local auth/session/RBAC, and MySQL persistence.
- Mapped implementation areas: Rocket route modules (`auth/users/submissions/orders/reviews/cases/payments/content/admin`), Dioxus pages (`profile/submissions/orders/reviews/cases`), RBAC middleware + role seeds, SQL schema/migrations, and test suites.
- Rerun focus: previously flagged gaps (logout wiring, reconciliation generation, guided templates, docs consistency, RBAC boundaries).

4. Section-by-section Review

4.1 Hard Gates
- 1.1 Documentation and static verifiability
  - Conclusion: Partial Pass
  - Rationale: Startup/config/test instructions exist and project structure is statically navigable, but API documentation is inconsistent across docs and contains wrong endpoints/methods in root spec.
  - Evidence: `repo/README.md:3`, `repo/README.md:23`, `README.md:32`, `docs/api-spec.md:144`, `docs/api-spec.md:213`, `docs/api-spec.md:345`, `repo/backend/src/routes/submissions.rs:720`, `repo/backend/src/routes/reviews.rs:63`, `repo/backend/src/routes/content.rs:172`
  - Manual verification note: Not needed for this conclusion (static mismatch).
- 1.2 Material deviation from prompt
  - Conclusion: Partial Pass
  - Rationale: Core domain flows exist; however, frontend API base URL is hardcoded to localhost, conflicting with prompt requirement to support same local network or localhost deployments.
  - Evidence: `repo/frontend/src/services/api.rs:5`, `repo/docker-compose.yml:63`
  - Manual verification note: Confirm on a second LAN device.

4.2 Delivery Completeness
- 2.1 Coverage of explicit core requirements
  - Conclusion: Partial Pass
  - Rationale: Most explicit backend capabilities are present (guided templates, version limits, watermark headers, reconciliation records, follow-up/image limits, SLA fields), but some required behavior remains under-evidenced in integrated UX/docs consistency.
  - Evidence: `repo/backend/src/routes/submissions.rs:328`, `repo/backend/src/routes/submissions.rs:522`, `repo/backend/src/routes/submissions.rs:649`, `repo/backend/src/routes/orders.rs:597`, `repo/backend/src/routes/reviews.rs:81`, `repo/backend/src/routes/reviews.rs:181`, `repo/backend/src/routes/cases.rs:83`
  - Manual verification note: Watermark visibility in actual opened files requires manual validation.
- 2.2 0→1 deliverable vs partial/demo
  - Conclusion: Pass
  - Rationale: Multi-module backend/frontend, migrations, docs, and test suites are present; no single-file demo pattern.
  - Evidence: `repo/Cargo.toml:1`, `repo/backend/src/main.rs:70`, `repo/frontend/src/main.rs:12`, `repo/backend/src/migrations/001_initial.sql:1`, `repo/API_tests/src/lib.rs:1`, `repo/unit_tests/src/lib.rs:1`

4.3 Engineering and Architecture Quality
- 3.1 Structure and decomposition
  - Conclusion: Pass
  - Rationale: Route separation, middleware/auth guard, and model modules are appropriately decomposed for scale.
  - Evidence: `repo/backend/src/routes/mod.rs:1`, `repo/backend/src/middleware/auth_guard.rs:16`, `repo/backend/src/models/mod.rs:1`
- 3.2 Maintainability/extensibility
  - Conclusion: Partial Pass
  - Rationale: Architecture is generally extensible, but duplicated/competing API specs increase maintenance risk and verification friction.
  - Evidence: `docs/api-spec.md:1`, `repo/docs/api-spec.md:1`

4.4 Engineering Details and Professionalism
- 4.1 Error handling/logging/validation/API shape
  - Conclusion: Partial Pass
  - Rationale: Input validation and status handling are present in key flows, but logging coverage is uneven and many failures are mapped without contextual logs.
  - Evidence: `repo/backend/src/routes/submissions.rs:343`, `repo/backend/src/routes/orders.rs:385`, `repo/backend/src/routes/reviews.rs:188`, `repo/backend/src/main.rs:106`, `repo/backend/src/routes/orders.rs:208`
- 4.2 Product-level shape vs demo
  - Conclusion: Pass
  - Rationale: Includes role-aware navigation, persistence model, and multi-domain workflows typical of product code.
  - Evidence: `repo/frontend/src/components/nav.rs:16`, `repo/frontend/src/pages/profile.rs:197`, `repo/frontend/src/pages/orders.rs:103`, `repo/frontend/src/pages/cases.rs:37`

4.5 Prompt Understanding and Requirement Fit
- 5.1 Business goal and constraint fit
  - Conclusion: Partial Pass
  - Rationale: Business workflows are substantially implemented, but LAN/localhost flexibility requirement is not met in frontend transport config; documentation accuracy gaps also weaken requirement fit.
  - Evidence: `repo/frontend/src/services/api.rs:5`, `docs/api-spec.md:213`, `repo/backend/src/routes/reviews.rs:63`

4.6 Aesthetics (frontend)
- 6.1 Visual/interaction quality
  - Conclusion: Cannot Confirm Statistically
  - Rationale: Static code shows structured pages/tables/forms and status badges, but visual fidelity, spacing correctness, and interaction feedback quality require runtime rendering.
  - Evidence: `repo/frontend/src/pages/orders.rs:165`, `repo/frontend/src/pages/reviews.rs:231`, `repo/frontend/src/pages/cases.rs:51`, `repo/frontend/assets/style.css:1`
  - Manual verification note: Browser review on desktop/mobile required.

5. Issues / Suggestions (Severity-Rated)

- Severity: High
- Title: Frontend transport hardcoded to localhost blocks LAN deployment scenario from prompt
- Conclusion: Fail
- Evidence: `repo/frontend/src/services/api.rs:5`, `repo/docker-compose.yml:63`
- Impact: Portal cannot be reliably consumed from another device on the same local network without code change; this conflicts with required localhost-or-LAN operation.
- Minimum actionable fix: Make backend base URL configurable (env/config/runtime setting) and ensure Dioxus frontend reads it instead of fixed literal.

- Severity: High
- Title: Root API spec contains incorrect endpoints/methods versus implementation
- Conclusion: Fail
- Evidence: `docs/api-spec.md:144`, `docs/api-spec.md:213`, `docs/api-spec.md:345`, `repo/backend/src/routes/submissions.rs:720`, `repo/backend/src/routes/reviews.rs:63`, `repo/backend/src/routes/content.rs:172`
- Impact: Hard-gate documentation verifiability is degraded; reviewers/integrators can call wrong routes and incorrectly judge delivery status.
- Minimum actionable fix: Align `docs/api-spec.md` with actual route signatures and keep only one source-of-truth API spec or generate it from code.

- Severity: Medium
- Title: RBAC seed grants `users.manage` to academic staff despite admin-only user deactivation policy
- Conclusion: Partial Fail
- Evidence: `repo/backend/src/migrations/002_seed.sql:56`, `repo/backend/src/routes/users.rs:124`
- Impact: Over-privileged baseline increases policy drift risk and future misuse when new user-management endpoints are added.
- Minimum actionable fix: Remove `perm-users-manage` from academic staff seed mapping unless explicitly required by policy; document intended privilege matrix.

- Severity: Medium
- Title: Security-critical flow coverage gaps remain in test suites
- Conclusion: Partial Fail
- Evidence: `repo/API_tests/src/lib.rs:98`, `repo/API_tests/src/lib.rs:657`, `repo/API_tests/src/lib.rs:758`, `repo/unit_tests/src/lib.rs:82`
- Impact: Severe regressions in logout invalidation, template endpoint behavior, and reconciliation generation could pass CI undetected.
- Minimum actionable fix: Add API tests for `/api/auth/logout`, `/api/submissions/templates`, reconciliation creation/update semantics, and role-change/audit-path assertions.

- Severity: Low
- Title: Duplicate API specs (`docs/` vs `repo/docs/`) increase drift risk
- Conclusion: Partial Fail
- Evidence: `docs/api-spec.md:1`, `repo/docs/api-spec.md:1`
- Impact: Ongoing inconsistency and reviewer confusion.
- Minimum actionable fix: Keep one canonical spec file and reference it from both READMEs.

6. Security Review Summary
- Authentication entry points: Pass
  - Evidence: `repo/backend/src/routes/auth.rs:75`, `repo/backend/src/routes/auth.rs:463`, `repo/backend/src/middleware/auth_guard.rs:54`
  - Reasoning: JWT + DB-backed active session check + idle refresh/expiry are implemented.
- Route-level authorization: Partial Pass
  - Evidence: `repo/backend/src/routes/users.rs:11`, `repo/backend/src/routes/orders.rs:220`, `repo/backend/src/routes/cases.rs:147`
  - Reasoning: Most sensitive routes enforce permission/privileged checks; policy seeding still shows over-broad mapping risk.
- Object-level authorization: Pass
  - Evidence: `repo/backend/src/routes/submissions.rs:588`, `repo/backend/src/routes/orders.rs:170`, `repo/backend/src/routes/cases.rs:190`, `repo/backend/src/routes/reviews.rs:149`
  - Reasoning: Owner/privileged checks are present in major IDOR-prone endpoints.
- Function-level authorization: Partial Pass
  - Evidence: `repo/backend/src/routes/users.rs:100`, `repo/backend/src/routes/users.rs:124`, `repo/backend/src/migrations/002_seed.sql:56`
  - Reasoning: Function permissions exist, but seeded permission scope suggests least-privilege misalignment.
- Tenant/user data isolation: Pass
  - Evidence: `repo/backend/src/routes/users.rs:29`, `repo/backend/src/routes/orders.rs:31`, `repo/backend/src/routes/submissions.rs:518`
  - Reasoning: Resource ownership checks are present for user/profile/order/submission flows.
- Admin/internal/debug endpoint protection: Pass
  - Evidence: `repo/backend/src/routes/admin.rs:11`, `repo/backend/src/routes/admin.rs:41`, `repo/backend/src/routes/admin.rs:74`
  - Reasoning: Admin endpoints require admin permissions; no open debug route observed.

7. Tests and Logging Review
- Unit tests: Partial Pass
  - Evidence: `repo/unit_tests/src/lib.rs:1`
  - Reasoning: Constants/validators are covered, but several tests are simplistic and not tied to full route behavior.
- API/integration tests: Partial Pass
  - Evidence: `repo/API_tests/src/lib.rs:1`, `repo/API_tests/src/lib.rs:248`, `repo/API_tests/src/lib.rs:758`, `repo/API_tests/src/lib.rs:1124`
  - Reasoning: Good breadth for RBAC/IDOR/watermark/review limits, but missing tests for newly changed flows (logout/templates/reconciliation generation details).
- Logging categories/observability: Partial Pass
  - Evidence: `repo/backend/src/main.rs:19`, `repo/backend/src/main.rs:106`, `repo/backend/src/routes/orders.rs:208`, `repo/backend/src/middleware/auth_guard.rs:93`
  - Reasoning: Uses structured log levels in key paths, but many error paths still return generic 500 without contextual logs.
- Sensitive-data leakage risk in logs/responses: Pass
  - Evidence: `repo/backend/src/routes/auth.rs:93`, `repo/backend/src/routes/submissions.rs:664`
  - Reasoning: No password/token plaintext logging observed; audit records include operational metadata but not secret credentials.

8. Test Coverage Assessment (Static Audit)

8.1 Test Overview
- Unit tests exist: Yes (`repo/unit_tests/src/lib.rs`)
- API/integration tests exist: Yes (`repo/API_tests/src/lib.rs`)
- Frameworks: Rust `#[test]` and `#[tokio::test]` with `reqwest`
- Test entry points: `cargo test -p unit_tests`, `cargo test -p API_tests`, `./run_tests.sh`
- Test commands documented: Yes
- Evidence: `repo/unit_tests/src/lib.rs:1`, `repo/API_tests/src/lib.rs:1`, `repo/README.md:23`, `repo/run_tests.sh:52`

8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Unauthenticated access returns 401 | `repo/API_tests/src/lib.rs:108` | GET `/api/auth/me` expects 401 (`:112`) | basically covered | Narrow endpoint sample only | Add matrix for core protected routes (`/api/orders`, `/api/submissions`, `/api/cases`) |
| RBAC denial for non-privileged users | `repo/API_tests/src/lib.rs:157`, `:168`, `:179`, `:194` | 403 assertions across admin/users/payments/content | sufficient | Does not cover role-change/deactivation path | Add tests for `/api/users/<id>/role` and `DELETE /api/users/<id>` role boundaries |
| Object-level authorization (IDOR) | `repo/API_tests/src/lib.rs:248`, `:282`, `:327`, `:519` | Cross-user access/comment attempts denied | sufficient | No explicit IDOR tests for reconciliation endpoints | Add cross-user tests for `/api/orders/<id>/reconciliation` and updates |
| Submission metadata validation | `repo/API_tests/src/lib.rs:219`, `:577` | 422 for long title/summary | sufficient | No boundary tests for tags/keywords lengths | Add API tests for tags/keywords max and over-limit |
| File download watermark contract | `repo/API_tests/src/lib.rs:758`, `:842` | Native content-type, watermark headers, non-ZIP body checks | sufficient | No DOCX/JPG watermark contract test | Add DOCX/JPG download assertions |
| Review follow-up/image constraints | `repo/API_tests/src/lib.rs:974`, `:1031`, `:1076`, `:1124` | one-followup rule, no nested followup, image type + max6 | sufficient | No explicit >5MB payload case | Add payload-too-large image test (413) |
| Session/logout invalidation | No dedicated API test found | N/A | missing | Regressions in logout/session invalidation may pass | Add login→logout→protected-route must return 401 |
| Guided template endpoint | No dedicated API test found | N/A | missing | Template listing/shape regressions undetected | Add GET `/api/submissions/templates` schema/value test |
| Reconciliation generation semantics | `repo/API_tests/src/lib.rs:657` | Only auth guard tested for reconciliation endpoint | insufficient | No assertion that records are auto-created/updated | Add create-order + fulfillment event tests validating expected/received/status transitions |
| After-sales SLA/status workflow | Partial: case ownership/type tests (`:327`, `:363`) | Authorization/type checks only | insufficient | No coverage of status transition sequence and SLA timestamps | Add privileged transition tests submitted→...→closed with invalid transition rejection |

8.3 Security Coverage Audit
- Authentication: Basically covered
  - Evidence: login invalid creds and unauthorized access tests (`repo/API_tests/src/lib.rs:99`, `:108`)
  - Gap: no logout invalidation/session-timeout API test.
- Route authorization: Basically covered
  - Evidence: several 403 tests (`repo/API_tests/src/lib.rs:157`, `:168`, `:179`)
  - Gap: missing coverage for some sensitive admin/user-management routes.
- Object-level authorization: Sufficient
  - Evidence: submission/order/case/comment IDOR tests (`repo/API_tests/src/lib.rs:248`, `:282`, `:327`, `:519`)
  - Gap: reconciliation object-level paths not covered.
- Tenant/data isolation: Basically covered
  - Evidence: cross-user access denials across multiple domains (`repo/API_tests/src/lib.rs:272`, `:303`, `:419`)
  - Gap: limited checks on list filtering beyond sampled endpoints.
- Admin/internal protection: Basically covered
  - Evidence: student denied admin dashboard (`repo/API_tests/src/lib.rs:157`)
  - Gap: no direct tests for all admin endpoints (`audit-log`, `settings`, cleanup).

8.4 Final Coverage Judgment
- Partial Pass
- Major risks covered: core auth denial cases, key RBAC denials, multiple IDOR scenarios, review/image constraints, and download watermark contract.
- Major uncovered risks: logout/session invalidation, templates endpoint correctness, reconciliation generation/update semantics, and full admin-route protection matrix; severe defects in these areas could still escape tests.

9. Final Notes
- Rerun confirms several prior issues are fixed: frontend logout now calls backend logout, submissions templates endpoint and UI wiring exist, and order reconciliation records are now generated/updated in backend logic.
- Remaining material risks are primarily requirement-fit (LAN transport config), documentation correctness, and high-impact coverage gaps.

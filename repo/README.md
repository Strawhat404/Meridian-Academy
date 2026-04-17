# Meridian Academic Publishing & Fulfillment Portal

**Project Type:** Full-Stack Web Application (Rust backend + Rust/WASM frontend)

A full-stack, offline-first web application for academic institutions managing content submissions, journal order fulfillment, peer reviews, and after-sales case management.

## Quick Start

```bash
cd repo
docker-compose up
```

| Service  | URL                      |
|----------|--------------------------|
| Backend  | http://localhost:8000    |
| Frontend | http://localhost:8080    |

> No configuration needed — all defaults are built in. Optionally copy `.env.example` to `.env` to customize credentials.

## Demo Credentials

All four roles are pre-seeded with direct login credentials. No provisioning required.

| Role              | Username      | Password   | Capabilities                                                     |
|-------------------|---------------|------------|------------------------------------------------------------------|
| Administrator     | `admin`       | `admin123` | Full access: user management, payments, audit, settings          |
| Academic Staff    | `staff1`      | `admin123` | Manage orders/cases/payments, review content, list users         |
| Instructor        | `instructor1` | `admin123` | Create/review submissions, create orders/reviews/cases           |
| Student           | `student1`    | `admin123` | Create submissions, orders, reviews, cases (own resources only)  |

> All demo accounts share the default password set by the `ADMIN_PASSWORD` environment variable (default: `admin123`). Override it in `.env` before first boot to change all demo passwords at once.

## Verification

After `docker-compose up` completes and both services are healthy:

**1. Health check (no auth required):**

```bash
curl http://localhost:8000/health
# Expected: {"status":"ok"}
```

**2. Admin login:**

```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
# Expected: 200 with {"token":"...","user":{"role":"administrator",...}}
```

**3. Admin dashboard (authenticated):**

```bash
curl http://localhost:8000/api/admin/dashboard \
  -H "Authorization: Bearer <token_from_step_2>"
# Expected: 200 with {"total_users":...,"total_orders":...,...}
```

**4. Frontend UI:**

Open http://localhost:8080 in a browser. Sign in with `admin` / `admin123`. You should see the Dashboard with summary cards.

## Roles & Permissions

The system enforces role-based access control (RBAC). Each user has exactly one role, and permissions are loaded from the database on every request (never cached from the JWT).

| Role              | Key Permissions                                                                                      |
|-------------------|------------------------------------------------------------------------------------------------------|
| `student`         | Create submissions, orders, reviews, cases. View own resources only.                                 |
| `instructor`      | Everything students can do, plus review/approve submissions from others.                             |
| `academic_staff`  | List all users, manage orders (split/merge/fulfill), manage cases, manage payments, review content.  |
| `administrator`   | Everything above, plus: provision users, change roles, audit log, system settings, reset tokens.     |

**Key security behaviors:**
- Sessions use a 30-minute idle timeout with server-side validation (fail-closed).
- Role changes immediately invalidate all existing sessions for that user.
- Deactivated users are denied on their next request (the auth guard re-checks the DB).
- Password reset tokens expire after 60 minutes and are single-use.
- Soft-deleted accounts have a 30-day recovery window before permanent deletion.

## Running Tests

```bash
cd repo
./run_tests.sh
```

The test suite includes three crates:

| Crate             | Type               | Description                                                      |
|-------------------|--------------------|------------------------------------------------------------------|
| `unit_tests`      | Unit tests         | Domain rules, business invariants, auth/security logic           |
| `API_tests`       | Integration (HTTP) | End-to-end flows against a running backend (requires backend up) |
| `frontend_tests`  | Contract + UI logic| DTO serialization, UI helper functions, RBAC display rules       |

**Expected outcome:** All tests print `RESULT: ALL TESTS PASSED` on exit code 0.

**Prerequisites for API tests:** The backend and database must be running (`docker-compose up`) before executing `./run_tests.sh`. Unit and frontend tests run without a live server.

**Troubleshooting:**
- If API tests fail with "Backend must be reachable", ensure `docker-compose up` is running and the backend is healthy (`curl http://localhost:8000/health`).
- If tests fail with DB errors, wait 10-15 seconds after `docker-compose up` for MySQL to initialize.
- The script auto-detects local `cargo` or falls back to a Docker-based Rust toolchain.

## Project Structure

```
Meridian_Academy/
├── docs/
│   ├── design.md            # System architecture and design
│   ├── api-spec.md          # REST API endpoint specifications
│   └── questions.md         # Clarifying questions and answers
├── repo/
│   ├── Cargo.toml           # Workspace manifest
│   ├── backend/             # Rocket REST API (Rust)
│   │   ├── src/
│   │   │   ├── main.rs      # Server bootstrap, route mounting, DB pool
│   │   │   ├── lib.rs       # Library root — exposes models for test crates
│   │   │   ├── routes/      # Route handlers grouped by domain
│   │   │   │   ├── auth.rs        # Login, provision, password reset, logout, export
│   │   │   │   ├── users.rs       # Profile, addresses, notifications, roles
│   │   │   │   ├── submissions.rs # CRUD, versioning, file upload/download
│   │   │   │   ├── orders.rs      # CRUD, split/merge, fulfillment, reconciliation
│   │   │   │   ├── reviews.rs     # Create, follow-up, images
│   │   │   │   ├── cases.rs       # After-sales lifecycle with SLA tracking
│   │   │   │   ├── payments.rs    # Charges, refunds, idempotency, reconciliation
│   │   │   │   ├── content.rs     # Sensitive words, content governance lifecycle
│   │   │   │   ├── admin.rs       # Dashboard stats, audit log, settings, cleanup
│   │   │   │   └── health.rs      # Health check
│   │   │   ├── models/      # Domain data structures (shared with test crates)
│   │   │   ├── middleware/   # JWT auth guard (fail-closed, DB-backed sessions)
│   │   │   ├── notifications.rs  # Preference-aware notification creation
│   │   │   └── migrations/  # SQL schema + seed data
│   │   └── Dockerfile
│   ├── frontend/             # Dioxus WASM frontend (Rust)
│   │   ├── src/
│   │   │   ├── main.rs       # App root, routing, page components
│   │   │   ├── components/   # Shared UI components (nav, layout)
│   │   │   ├── pages/        # Page components (admin, cases, orders, etc.)
│   │   │   └── services/     # API client, auth (token storage)
│   │   └── Dockerfile
│   ├── unit_tests/           # Pure unit tests (no live server needed)
│   │   └── src/
│   │       ├── lib.rs          # Core domain + constant tests
│   │       ├── domain_rules.rs # Business logic edge cases
│   │       └── security.rs     # Auth, RBAC, crypto invariants
│   ├── API_tests/            # HTTP integration tests (live server required)
│   │   └── src/
│   │       ├── lib.rs          # IDOR, RBAC, validation, reconciliation
│   │       ├── flows.rs        # End-to-end user journeys + all endpoint coverage
│   │       └── security.rs     # Auth enforcement, data-leak prevention
│   ├── frontend_tests/       # Frontend contract + UI logic tests
│   │   └── src/
│   │       ├── lib.rs          # DTO serialization, RBAC display, path construction
│   │       ├── ui_logic.rs     # Badge mapping, formatting, pagination, menus
│   │       └── contracts.rs    # JSON round-trip for every backend DTO
│   ├── docker-compose.yml
│   ├── run_tests.sh
│   └── .env.example
```

## Architecture

```
┌──────────────┐     HTTP/JSON     ┌──────────────────┐     MySQL     ┌─────────┐
│  Dioxus WASM │ ◄──────────────► │   Rocket Backend  │ ◄──────────► │ MySQL 8 │
│  (port 8080) │                   │   (port 8000)     │              │         │
└──────────────┘                   └──────────────────┘              └─────────┘
     Browser                        JWT + DB sessions                  Schema +
                                    RBAC auth guard                    seed data
```

- **Frontend** compiles to WebAssembly via Trunk. Runs entirely in the browser; no SSR.
- **Backend** is a stateless REST API. Auth uses JWT tokens validated against server-side sessions stored in MySQL. Every request re-checks the user's role and active status from the database (fail-closed).
- **Database** is MySQL 8. Schema and seed data are applied automatically on first boot via embedded migrations.
- **Offline-first**: No external API calls, no CDN, no cloud services. Runs on an air-gapped LAN.

## Tech Stack

- **Frontend**: Dioxus 0.5 (Rust → WebAssembly via Trunk)
- **Backend**: Rocket (Rust REST API)
- **Database**: MySQL 8
- **Auth**: Bcrypt password hashing, JWT tokens, DB-backed sessions, 30-min idle timeout
- **Offline**: Fully runnable on local network, no external services required

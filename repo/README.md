# Meridian Academic Publishing & Fulfillment Portal

A full-stack, offline-first web application for academic institutions managing content submissions, journal order fulfillment, peer reviews, and after-sales case management.




## Quick Start

```bash
cd repo
docker compose up
# Backend: http://localhost:8000
# Frontend: http://localhost:8080
#Username: admin
#Passsword: admin123
```

> No configuration needed — all defaults are built in. Optionally copy `.env.example` to `.env` to customize credentials.

## Running Tests

```bash
cd repo
./run_tests.sh
```

## Project Structure

```
Meridian_Academy/
├── docs/
│   ├── design.md        # System architecture and design
│   ├── api-spec.md      # REST API endpoint specifications
│   └── questions.md     # Clarifying questions and answers
├── repo/
│   ├── Cargo.toml       # Workspace manifest
│   ├── backend/         # Rocket REST API (Rust)
│   │   ├── src/
│   │   │   ├── routes/  # auth, users, orders, submissions, reviews, cases, payments, content, admin
│   │   │   ├── models/  # domain models
│   │   │   ├── middleware/
│   │   │   └── migrations/
│   │   └── Dockerfile
│   ├── frontend/        # Dioxus WASM frontend (Rust)
│   │   ├── src/
│   │   └── Dockerfile
│   ├── unit_tests/      # Rust unit tests
│   ├── API_tests/       # Rust integration tests
│   ├── docker-compose.yml
│   ├── run_tests.sh
│   └── .env.example
```

## Tech Stack

- **Frontend**: Dioxus (Rust → WebAssembly via Trunk)
- **Backend**: Rocket (Rust REST API)
- **Database**: MySQL 8
- **Auth**: Local-only, salted password hashing, 30-min idle session timeout
- **Offline**: Fully runnable on local network, no external services required

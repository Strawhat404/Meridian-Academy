# Meridian Academy — Repository

## Local Run (without Docker)

**Prerequisites:** Rust toolchain, `trunk` (`cargo install trunk`), MySQL 8 running locally.

```bash
# 1. Create the database
mysql -u root -p -e "CREATE DATABASE meridian_academy; CREATE USER 'meridian_user'@'localhost' IDENTIFIED BY 'meridian_pass'; GRANT ALL ON meridian_academy.* TO 'meridian_user'@'localhost';"

# 2. Configure environment
cp .env.example .env
# Edit .env — set DATABASE_URL=mysql://meridian_user:meridian_pass@localhost:3306/meridian_academy

# 3. Run backend (migrations + seed run automatically on startup)
cargo run -p backend

# 4. In a separate terminal, run frontend
cd frontend && trunk serve
# Frontend: http://localhost:8080  Backend: http://localhost:8000
```

**Running tests:**
```bash
# Unit tests — no backend needed
cargo test -p unit_tests

# API/integration tests — requires running backend + seeded DB
# Default seed credentials: username=admin password=admin123 (or ADMIN_PASSWORD env var)
cargo test -p API_tests
```

## Quick Start (Docker)

```bash
# Start all services (no configuration needed — defaults are built in)
docker compose up

# Backend API: http://localhost:8000
# Frontend UI: http://localhost:8080
```

> Optional: copy `.env.example` to `.env` to customize credentials before starting.

## Running Tests

```bash
./run_tests.sh
```

Or manually:

```bash
cargo test -p unit_tests
cargo test -p API_tests
```

## Environment Variables

See `.env.example` for all required variables including `DATABASE_URL`, `ROCKET_PORT`, session timeout, and token expiry settings.

## Services

| Service | Port | Description |
|---------|------|-------------|
| MySQL | 3306 | Primary database |
| Backend (Rocket) | 8000 | REST API |
| Frontend (Dioxus) | 8080 | Web UI |

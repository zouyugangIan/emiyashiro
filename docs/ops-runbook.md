# G-Engine Ops Runbook

> Last verified: 2026-02-24  
> Scope: local/dev and single-node staging operations for `client` + `server`.

## 1. Preconditions

- Rust toolchain installed (edition 2024 compatible, MSRV in `Cargo.toml`).
- Docker and Docker Compose available.
- Ports available: `5432`, `5672`, `6379`, `8080`, `15672`.

## 2. Service Startup

### Infrastructure

```bash
docker-compose up -d
```

### Environment

```bash
export REDIS_URL="redis://127.0.0.1:6379/"
export RABBITMQ_URL="amqp://guest:guest@127.0.0.1:5672/%2f"
export DATABASE_URL="postgresql://username:password@localhost/shirou_runner"
```

### Application

```bash
cargo run --bin server --features server
cargo run --bin client --features client
```

## 3. Health Checks

### Build and test gate

```bash
cargo fmt --check
cargo check
cargo check --all-features --future-incompat-report
cargo clippy --lib --all-features -- -D warnings
cargo clippy --all-features --all-targets -- -D warnings
cargo test --lib --all-features
```

### Runtime checks

```bash
# Redis
redis-cli ping

# RabbitMQ management API
curl -sS http://127.0.0.1:15672 >/dev/null

# PostgreSQL
psql -h 127.0.0.1 -U username -d shirou_runner -c "select now();"
```

Expected:

- Server logs show `WebSocket server listening on: 127.0.0.1:8080`.
- Client receives `Welcome` and world snapshots.
- Redis keys such as `player:1:pos` can be read.

## 4. Incident Playbooks

### A. Server cannot connect to PostgreSQL

1. Check container status:
   - `docker ps | rg postgres`
2. Validate credentials in `DATABASE_URL`.
3. Verify DB connectivity:
   - `psql -h 127.0.0.1 -U username -d shirou_runner -c "select 1;"`
4. Restart only postgres:
   - `docker compose restart postgres`

### B. Save worker not consuming `q_save_game`

1. Check queue in RabbitMQ UI (`http://127.0.0.1:15672`).
2. Verify `RABBITMQ_URL`.
3. Restart RabbitMQ:
   - `docker compose restart rabbitmq`
4. Restart server process.

### C. Redis sync degraded

1. Check Redis:
   - `redis-cli ping`
2. Confirm key updates:
   - `GET player:9999:pos`
3. Inspect server stderr for Redis write errors.
4. Restart Redis:
   - `docker compose restart redis`

### D. WebSocket clients cannot connect

1. Confirm server running on `127.0.0.1:8080`.
2. Check local firewall / port conflict:
   - `ss -ltnp | rg 8080`
3. Verify client URL in `src/systems/network.rs`.

## 5. Backup and Restore (PostgreSQL)

### Backup

```bash
docker exec -t $(docker ps --format '{{.Names}}' | rg postgres -m1) \
  pg_dump -U username -d shirou_runner > backup_shirou_runner.sql
```

### Restore

```bash
cat backup_shirou_runner.sql | docker exec -i $(docker ps --format '{{.Names}}' | rg postgres -m1) \
  psql -U username -d shirou_runner
```

## 6. Rollback Procedure

1. Identify target commit/tag.
2. Stop client/server processes.
3. Checkout target revision.
4. Run quality gate in section 3.
5. Restart infra and app.

## 7. Release Readiness Checklist

- [ ] CI green on `.github/workflows/rust-ci.yml`
- [ ] `cargo check --all-features --future-incompat-report` shows no future incompat warnings
- [ ] `cargo test --lib --all-features` passes
- [ ] Infrastructure services healthy
- [ ] Rollback commit identified

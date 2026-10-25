SHELL := /bin/bash
# technical: task-bounded PostgreSQL and server readiness waits.
E2E_STARTUP_TIMEOUT_SECONDS ?= 60
# technical: use the supported browser runtime for local E2E by default.
E2E_PLAYWRIGHT_CONTAINER ?= 1

.PHONY: dev-db dev test lint api e2e build clean

dev-db:
	docker compose -f deploy/dev/compose.yaml up -d

dev:
	@if [[ ! -f .env ]]; then \
	  echo 'Create .env from .env.example and fill in the development values.' >&2; \
	  exit 1; \
	fi
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
	  echo 'Install the optional watcher with: cargo install --locked cargo-watch' >&2; \
	  echo 'Manual fallback, in separate terminals:' >&2; \
	  echo '  set -a; source .env; set +a; cargo run -p eq-server' >&2; \
	  echo '  pnpm -C web dev' >&2; \
	  exit 1; \
	fi
	set -a; source .env; set +a; \
	cargo watch -x 'run -p eq-server' & server_pid=$$!; \
	pnpm -C web dev & web_pid=$$!; \
	trap 'kill "$$server_pid" "$$web_pid" 2>/dev/null || true' EXIT INT TERM; \
	wait -n "$$server_pid" "$$web_pid"

test:
	cargo test --workspace
	pnpm -C web test

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	pnpm -C web lint
	pnpm -C web typecheck
	scripts/check-deps.sh
	scripts/check-literals.sh
	scripts/check-web-boundaries.sh
	scripts/check-layers.sh

api:
	@echo 'OpenAPI generation is added by T-114.'
	@false

e2e:
	@set -Eeuo pipefail; \
	if [[ "$${E2E_SKIP_DB_START:-0}" != "1" ]]; then \
	  $(MAKE) dev-db; \
	  db_ready=0; \
	  for ((attempt=0; attempt<$(E2E_STARTUP_TIMEOUT_SECONDS); attempt++)); do \
	    if docker compose -f deploy/dev/compose.yaml exec -T db pg_isready -U eq -d ritma >/dev/null 2>&1; then db_ready=1; break; fi; \
	    sleep 1; \
	  done; \
	  if [[ "$$db_ready" != "1" ]]; then echo 'PostgreSQL did not become ready for E2E.' >&2; exit 1; fi; \
	fi; \
	export DATABASE_URL="$${DATABASE_URL:-postgres://eq_app:eq_app@localhost:5433/ritma}"; \
	export RITMA_DATABASE_OWNER_URL="$${RITMA_DATABASE_OWNER_URL:-postgres://eq:eq@localhost:5433/ritma}"; \
	export RITMA_SECRET_KEY="$${RITMA_SECRET_KEY:-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=}"; \
	export RITMA_PUBLIC_URL="$${RITMA_PUBLIC_URL:-http://localhost:8080}"; \
	export RITMA_LISTEN="$${RITMA_LISTEN:-127.0.0.1:8080}"; \
	export RITMA_BACKUP_SCHEDULE="$${RITMA_BACKUP_SCHEDULE:-off}"; \
	export E2E_BASE_URL="$${E2E_BASE_URL:-http://127.0.0.1:8080}"; \
	pnpm -C web build; \
	cargo build --release -p eq-server; \
	tmp_data_dir=""; \
	if [[ -z "$${RITMA_DATA_DIR:-}" ]]; then tmp_data_dir="$$(mktemp -d)"; export RITMA_DATA_DIR="$$tmp_data_dir"; fi; \
	mkdir -p "$$RITMA_DATA_DIR"; \
	server_log="$$(mktemp)"; \
	target/release/ritma serve >"$$server_log" 2>&1 & server_pid=$$!; \
	cleanup() { status=$$?; trap - EXIT; if kill -0 "$$server_pid" 2>/dev/null; then kill -TERM "$$server_pid" 2>/dev/null || true; fi; wait "$$server_pid" 2>/dev/null || true; if [[ "$$status" -ne 0 ]]; then cat "$$server_log" >&2; fi; rm -f "$$server_log"; if [[ -n "$$tmp_data_dir" ]]; then rm -rf "$$tmp_data_dir"; fi; exit "$$status"; }; \
	trap cleanup EXIT; \
	trap 'exit 130' INT; \
	trap 'exit 143' TERM; \
	server_ready=0; \
	for ((attempt=0; attempt<$(E2E_STARTUP_TIMEOUT_SECONDS); attempt++)); do \
	  if ! kill -0 "$$server_pid" 2>/dev/null; then break; fi; \
	  if curl -fsS "$$E2E_BASE_URL/readyz" >/dev/null 2>&1; then server_ready=1; break; fi; \
	  sleep 1; \
	done; \
	if [[ "$$server_ready" != "1" ]]; then echo 'Ritma did not become ready for E2E within the startup timeout.' >&2; exit 1; fi; \
	if [[ "$(E2E_PLAYWRIGHT_CONTAINER)" == "1" ]]; then \
	  docker run --rm --network host --user "$$(id -u):$$(id -g)" -v "$$PWD/web:/workspace" -w /workspace \
	    -e HOME=/tmp -e "E2E_BASE_URL=$$E2E_BASE_URL" -e PLAYWRIGHT_BROWSERS_PATH=/ms-playwright \
	    mcr.microsoft.com/playwright:v1.64.1-noble \
	    node_modules/.bin/playwright test --config=e2e/playwright.config.ts; \
	else \
	  pnpm -C web exec playwright test --config=e2e/playwright.config.ts; \
	fi

build:
	pnpm -C web build
	cargo build -p eq-server --release

clean:
	cargo clean
	rm -rf web/dist web/.vite web/test-results web/playwright-report

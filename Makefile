SHELL := /bin/bash

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
	@echo 'Playwright scenarios are added by T-008.'
	@false

build:
	pnpm -C web build
	cargo build -p eq-server --release

clean:
	cargo clean
	rm -rf web/dist web/.vite web/test-results web/playwright-report

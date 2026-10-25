#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
guard="${repository_root}/scripts/check-layers.sh"
fixture_root="$(mktemp -d)"
trap 'rm -rf "$fixture_root"' EXIT

pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; exit 1; }
reset_fixture() { rm -rf "$fixture_root/crates"; mkdir -p "$fixture_root/crates/eq-billing/src"; }

reset_fixture
mkdir -p "$fixture_root/crates/eq-billing/src/domain"
cat >"$fixture_root/crates/eq-billing/src/domain/invoice.rs" <<'RS'
use sqlx::PgPool;
RS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  fail "detects sqlx in domain (guard accepted it)"
elif [[ "$output" == *"crates/eq-billing/src/domain/invoice.rs:1"* ]]; then
  pass "detects sqlx in domain"
else
  printf '%s\n' "$output" >&2
  fail "detects sqlx in domain (missing path:line)"
fi

reset_fixture
cat >"$fixture_root/crates/eq-billing/src/repo.rs" <<'RS'
use axum::Json;
RS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  fail "detects axum in repo (guard accepted it)"
elif [[ "$output" == *"crates/eq-billing/src/repo.rs:1"* ]]; then
  pass "detects axum in repo"
else
  printf '%s\n' "$output" >&2
  fail "detects axum in repo (missing path:line)"
fi

reset_fixture
mkdir -p "$fixture_root/crates/eq-billing/src/domain"
cat >"$fixture_root/crates/eq-billing/src/domain/invoice.rs" <<'RS'
pub fn is_open() -> bool { true }
RS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  pass "passes clean crate"
else
  printf '%s\n' "$output" >&2
  fail "passes clean crate"
fi

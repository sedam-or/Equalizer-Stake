#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
guard="${repository_root}/scripts/check-literals.sh"
fixture_root="$(mktemp -d)"
trap 'rm -rf "$fixture_root"' EXIT

pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; exit 1; }
prepare_file() {
  rm -rf "$fixture_root/crates"
  mkdir -p "$fixture_root/crates/eq-billing/src/domain"
}

prepare_file
cat >"$fixture_root/crates/eq-billing/src/domain/x.rs" <<'RS'
let due = 10_000_000;
RS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  fail "detects money literal (guard accepted it)"
elif [[ "$output" == *"crates/eq-billing/src/domain/x.rs:1"* ]]; then
  pass "detects money literal"
else
  printf '%s\n' "$output" >&2
  fail "detects money literal (missing path:line)"
fi

prepare_file
cat >"$fixture_root/crates/eq-billing/src/domain/x.rs" <<'RS'
// technical: fixed capacity for the parser test fixture
const INTERNAL_BUFFER: usize = 10_000;
RS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  pass "ignores technical const"
else
  printf '%s\n' "$output" >&2
  fail "ignores technical const"
fi

prepare_file
cat >"$fixture_root/crates/eq-billing/src/domain/x.rs" <<'RS'
let timezone = "Asia/Jakarta";
RS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  fail "detects timezone string (guard accepted it)"
elif [[ "$output" == *"crates/eq-billing/src/domain/x.rs:1"* ]]; then
  pass "detects timezone string"
else
  printf '%s\n' "$output" >&2
  fail "detects timezone string (missing path:line)"
fi

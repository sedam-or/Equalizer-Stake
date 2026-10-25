#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
guard="${repository_root}/scripts/check-web-boundaries.sh"
fixture_root="$(mktemp -d)"
trap 'rm -rf "$fixture_root"' EXIT

pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; exit 1; }
reset_fixture() { rm -rf "$fixture_root/web"; mkdir -p "$fixture_root/web/src/modules/a" "$fixture_root/web/src/modules/b"; }

reset_fixture
cat >"$fixture_root/web/src/modules/a/page.ts" <<'TS'
import { view } from "@/modules/b/view";
TS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  fail "detects cross-module import (guard accepted it)"
elif [[ "$output" == *"web/src/modules/a/page.ts:1"* ]]; then
  pass "detects cross-module import"
else
  printf '%s\n' "$output" >&2
  fail "detects cross-module import (missing path:line)"
fi

reset_fixture
cat >"$fixture_root/web/src/modules/a/page.ts" <<'TS'
import { ownView } from "@/modules/a/view";
TS
cat >"$fixture_root/web/src/modules/registry.ts" <<'TS'
import { a } from "@/modules/a/view";
import { b } from "@/modules/b/view";
TS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  pass "allows registry"
else
  printf '%s\n' "$output" >&2
  fail "allows registry"
fi

reset_fixture
cat >"$fixture_root/web/src/modules/a/page.ts" <<'TS'
import { ownView } from "@/modules/a/view";
TS
if output="$("$guard" "$fixture_root" 2>&1)"; then
  pass "allows same-module import"
else
  printf '%s\n' "$output" >&2
  fail "allows same-module import"
fi

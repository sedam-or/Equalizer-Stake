#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd -- "${script_dir}/.." && pwd)"
if [[ $# -ne 1 || ! -f "$1" ]]; then
  printf 'usage: %s <lcov-file>\n' "$0" >&2
  exit 2
fi

python3 - "$root" "$1" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1]).resolve()
lcov = Path(sys.argv[2]).resolve()
domain_files = list((root / "crates").glob("*/src/domain/**/*.rs"))
domain_files.extend((root / "crates").glob("*/src/domain/*.rs"))
has_domain_source = any(path.is_file() for path in domain_files)

total = 0
covered = 0
in_domain = False
for line in lcov.read_text(encoding="utf-8").splitlines():
    if line.startswith("SF:"):
        source = line[3:].replace("\\", "/")
        in_domain = re.search(r"(?:^|/)crates/[^/]+/src/domain/", source) is not None
    elif in_domain and line.startswith("DA:"):
        fields = line[3:].split(",", 2)
        if len(fields) < 2:
            print(f"invalid LCOV DA record: {line}", file=sys.stderr)
            sys.exit(2)
        try:
            hits = int(fields[1])
        except ValueError:
            print(f"invalid LCOV hit count: {line}", file=sys.stderr)
            sys.exit(2)
        total += 1
        covered += hits > 0

if not has_domain_source:
    print("domain coverage gate passed: no domain source files exist")
    sys.exit(0)
if total == 0:
    print("domain coverage gate failed: LCOV contains no domain source records", file=sys.stderr)
    sys.exit(1)

percentage = covered * 100 / total
print(f"domain line coverage: {covered}/{total} ({percentage:.1f}%), required >= 90.0%")
if covered * 100 < 90 * total:
    sys.exit(1)
PY

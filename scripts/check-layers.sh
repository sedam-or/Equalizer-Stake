#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="${1:-$(cd -- "${script_dir}/.." && pwd)}"
if [[ $# -gt 1 ]]; then
  printf 'usage: %s [repository-root]\n' "$0" >&2
  exit 2
fi

python3 - "$root" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1]).resolve()
if not (root / "crates").is_dir():
    print(f"not a repository root: {root}", file=sys.stderr)
    sys.exit(2)

issues = []
for path in sorted((root / "crates").glob("*/src/**/*.rs")):
    relative = path.relative_to(root)
    parts = relative.parts
    source_index = parts.index("src")
    source_parts = parts[source_index + 1 :]
    text = path.read_text(encoding="utf-8")
    for line_number, line in enumerate(text.splitlines(), 1):
        if "domain" in source_parts[:-1]:
            for name in ("sqlx", "axum", "reqwest", "tokio", "tracing", "Clock", "Tx"):
                pattern = r"\b" + re.escape(name) + r"\b"
                if name == "std::env" or name == "std::fs":
                    continue
                if re.search(pattern, line):
                    issues.append((str(relative), line_number, f"{name} is forbidden in domain code"))
            for name in ("std::env", "std::fs"):
                if name in line:
                    issues.append((str(relative), line_number, f"{name} is forbidden in domain code"))

        if (source_parts == ("http.rs",) or source_parts[:1] == ("http",)) and re.search(r"\bsqlx\s*::", line):
            issues.append((str(relative), line_number, "sqlx is forbidden in HTTP adapters"))

        if (path.name == "repo.rs" or source_parts[:1] == ("repo",)) and re.search(r"\baxum\b", line):
            issues.append((str(relative), line_number, "axum is forbidden in repositories"))

for filename, line, reason in sorted(set(issues)):
    print(f"{filename}:{line}: {reason}")
sys.exit(1 if issues else 0)
PY

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="${1:-$(cd -- "${script_dir}/.." && pwd)}"
if [[ $# -gt 1 ]]; then
  printf 'usage: %s [repository-root]\n' "$0" >&2
  exit 2
fi

python3 - "$root" <<'PY'
from pathlib import Path, PurePosixPath
import os
import re
import sys

root = Path(sys.argv[1]).resolve()
modules_root = root / "web/src/modules"
if not modules_root.is_dir():
    sys.exit(0)

import_pattern = re.compile(
    r"\bfrom\s*[\"']([^\"']+)[\"']"
    r"|\bimport\s*[\"']([^\"']+)[\"']"
    r"|\bimport\s*\(\s*[\"']([^\"']+)[\"']\s*\)"
)
issues = []
extensions = {".ts", ".tsx", ".js", ".jsx", ".mts", ".cts"}
for path in sorted(modules_root.rglob("*")):
    if not path.is_file() or path.suffix not in extensions:
        continue
    relative = path.relative_to(root)
    module_relative = path.relative_to(modules_root)
    if module_relative.as_posix() == "registry.ts":
        continue
    source_module = module_relative.parts[0]
    text = path.read_text(encoding="utf-8")
    for match in import_pattern.finditer(text):
            line_number = text.count("\n", 0, match.start()) + 1
            specifier = next((item for item in match.groups() if item is not None), "")
            target_module = None
            if specifier.startswith("@/modules/"):
                parts = PurePosixPath(specifier).parts
                if len(parts) >= 3:
                    target_module = parts[2]
            elif specifier.startswith("."):
                resolved = os.path.normpath((path.parent / specifier).as_posix())
                try:
                    target_parts = PurePosixPath(resolved).relative_to(PurePosixPath(modules_root.as_posix())).parts
                except ValueError:
                    continue
                if target_parts:
                    target_module = target_parts[0]
            if target_module and target_module != source_module:
                issues.append((str(relative), line_number, f"module {source_module} imports module {target_module}"))

for filename, line, reason in sorted(set(issues)):
    print(f"{filename}:{line}: {reason}")
sys.exit(1 if issues else 0)
PY

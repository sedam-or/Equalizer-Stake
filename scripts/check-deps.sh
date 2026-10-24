#!/bin/sh
set -eu

repo_root=$(CDPATH= cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

optional_crates='["eq-enrollment", "eq-billing", "eq-attendance", "eq-rental", "eq-notify", "eq-reports", "eq-feedback"]'
violations=$(cargo metadata --format-version 1 --no-deps | jq -r --argjson optional "$optional_crates" '
  .packages[] as $package
  | $package.dependencies[]? as $dependency
  | select(
      ($package.name == "eq-core" and ($dependency.name | startswith("eq-")))
      or (
        ($optional | index($package.name)) != null
        and ($optional | index($dependency.name)) != null
      )
    )
  | "\($package.name) -> \($dependency.name)"
')

if [ -n "$violations" ]; then
    printf '%s\n%s\n' 'Forbidden crate dependency edges:' "$violations" >&2
    exit 1
fi

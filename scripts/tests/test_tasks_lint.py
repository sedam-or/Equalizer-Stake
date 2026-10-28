"""Fixtures for the task hygiene gate."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("tasks_lint", ROOT / "scripts" / "tasks-lint.py")
assert SPEC is not None and SPEC.loader is not None
tasks_lint = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = tasks_lint
SPEC.loader.exec_module(tasks_lint)


def task_text(*, status: str = "todo", dependency: str = "", body: str = "") -> str:
    return f"""---
id: T-001
title: Example task
milestone: M0
status: {status}
depends_on: [{dependency}]
spec_refs: []
estimated_size: S
---

## Goal
Do one thing.
## Context
The contract is inline.
## Scope
### In
Implement the stated contract.
### Out
Nothing else.
## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-core/src/example.rs` | modify | implementation |
## Interfaces
Use the contract above.
## Business rules
Use only values named above.
## Security
No triggers.
## Implementation notes
Follow the cited specs.
## Acceptance criteria
- AC1. Valid input succeeds. → `valid_input`
## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/example.rs` | `valid_input` |
## Definition of done
- [ ] checks pass
## Report
Summary and checks.
## Questions
n/a
## Review
n/a
{body}"""


class TasksLintTests(unittest.TestCase):
    def lint(self, documents: dict[str, str]) -> list[str]:
        with tempfile.TemporaryDirectory(dir=Path(__file__).parent) as directory:
            root = Path(directory)
            for relative, text in documents.items():
                path = root / "tasks" / "Fixture" / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding="utf-8")
            return tasks_lint.validate(root)

    def test_valid_fixture_passes(self) -> None:
        self.assertEqual([], self.lint({"T-001-example.md": task_text()}))

    def test_each_size_limit_rejects_oversized_fixture(self) -> None:
        too_many_files = task_text().replace(
            "| `crates/eq-core/src/example.rs` | modify | implementation |",
            "\n".join(f"| `crates/eq-core/src/file_{i}.rs` | modify | implementation |" for i in range(9)),
        )
        errors = self.lint({"T-001-example.md": too_many_files})
        self.assertTrue(any("Files rows exceeds 8" in error for error in errors))

        criteria = "\n".join(f"- AC{i}. Check behavior. → `test_{i}`" for i in range(1, 10))
        too_many_criteria = task_text().replace("- AC1. Valid input succeeds. → `valid_input`", criteria)
        errors = self.lint({"T-001-example.md": too_many_criteria})
        self.assertTrue(any("acceptance criteria exceeds 8" in error for error in errors))

        names = ", ".join(f"`test_{i}`" for i in range(13))
        too_many_tests = task_text().replace("`valid_input` |", f"{names} |")
        errors = self.lint({"T-001-example.md": too_many_tests})
        self.assertTrue(any("test names exceeds 12" in error for error in errors))

    def test_security_trigger_requires_section(self) -> None:
        triggered = task_text().replace("The contract is inline.", "A public login endpoint accepts a password.").replace(
            "## Security\nNo triggers.\n", ""
        )
        errors = self.lint({"T-001-example.md": triggered})
        self.assertTrue(any("security trigger without ## Security" in error for error in errors))

    def test_replaced_crate_is_rejected(self) -> None:
        replaced = task_text().replace("The contract is inline.", "Use phonenumber 0.3 for validation.")
        errors = self.lint({"T-001-example.md": replaced})
        self.assertTrue(any("replaced dependency" in error for error in errors))

    def test_unknown_dependency_is_rejected(self) -> None:
        errors = self.lint({"T-001-example.md": task_text(dependency="T-999")})
        self.assertTrue(any("unknown dependency T-999" in error for error in errors))

    def test_blocked_status_is_rejected(self) -> None:
        errors = self.lint({"T-001-example.md": task_text(status="blocked")})
        self.assertTrue(any("status blocked is not allowed" in error for error in errors))

    def test_missing_template_section_is_rejected(self) -> None:
        missing = task_text().replace("## Business rules\nUse only values named above.\n", "")
        errors = self.lint({"T-001-example.md": missing})
        self.assertTrue(any("missing required section ## Business rules" in error for error in errors))

    def test_acceptance_test_name_after_scenario_arrows_is_counted(self) -> None:
        document = task_text().replace(
            "- AC1. Valid input succeeds. → `valid_input`",
            "- AC1. A → B transition succeeds. → `valid_input`",
        ).replace(
            "| `crates/eq-core/tests/example.rs` | `valid_input` |",
            "| `crates/eq-core/tests/example.rs` | AC names above |",
        )
        self.assertEqual({"valid_input"}, tasks_lint._test_names(document))


if __name__ == "__main__":
    unittest.main()

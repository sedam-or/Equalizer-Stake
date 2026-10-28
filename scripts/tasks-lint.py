#!/usr/bin/env python3
"""Read-only validation for active task files."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


REQUIRED_SECTIONS = (
    "Goal",
    "Context",
    "Scope",
    "Files",
    "Interfaces",
    "Business rules",
    "Security",
    "Implementation notes",
    "Acceptance criteria",
    "Tests to write",
    "Definition of done",
    "Report",
    "Questions",
    "Review",
)
ACTIVE_STATUSES = {"todo", "draft"}
REPLACED_STACK = (
    (re.compile(r"(?i)(?<![a-z0-9_-])phonenumber\s*(?:=|\b(?:crate|version)\b|\s+0\.3)"), "phonenumber; use rlibphonenumber 2"),
    (re.compile(r"(?i)(?<![a-z0-9_-])utoipa\s*(?:=\s*[\"']?5\b|\b(?:version\s*)?5(?:\.\d+)?\b)"), "utoipa 5; use utoipa 6"),
    (re.compile(r"(?i)(?<![a-z0-9_-])utoipa-axum\s*(?:=\s*[\"']?0\.2\b|\b(?:version\s*)?0\.2(?:\.\d+)?\b)"), "utoipa-axum 0.2; use utoipa-axum 0.3"),
    (re.compile(r"(?i)(?:\bpaste\s*=\s*[\"']|\b(?:crate|dependency)\s+`?paste`?\b)"), "paste direct dependency"),
    (re.compile(r"(?i)(?:\batomic-polyfill\s*=\s*[\"']|\b(?:crate|dependency)\s+`?atomic-polyfill`?\b)"), "atomic-polyfill direct dependency"),
)
SECURITY_TRIGGER = re.compile(
    r"(?i)(?:\b(?:auth|authentication|login|password|credential|refresh[-_ ]?token|"
    r"invite[-_ ]?token|reset[-_ ]?token|totp|secret|crypto|encryption|key rotation|"
    r"RLS|tenant[-_ ](?:resolution|isolation)|tenant_id|read_pii|audit|permission|role matrix|"
    r"scope rule|upload|import|export|webhook|SMTP|WhatsApp|Stripe|Xendit|payment proof|"
    r"registration document|CSV|PDF|SECURITY DEFINER|cross[-_ ]tenant)\b|"
    r"\b(?:create|reschedule|cancel|confirm|approve)\b[^\n]{0,60}\b(?:session|booking|reservation|enrollment|payment)\b|"
    r"\b(?:session|booking|reservation|enrollment|payment)\b[^\n]{0,60}\b(?:create|reschedule|cancel|confirm|approve)\b)"
)
PATH_TOKEN = re.compile(
    r"(?<![A-Za-z0-9_.-])(?:crates|migrations|web|apps|scripts|presets|deploy)/"
    r"[A-Za-z0-9_.*{}./-]+|(?<![A-Za-z0-9_.-])(?:Cargo\.toml|Cargo\.lock|Makefile|CONTRIBUTING\.md|pnpm-lock\.yaml)"
)


@dataclass(frozen=True)
class Task:
    path: Path
    task_id: str
    status: str
    dependencies: tuple[str, ...]
    text: str


def _frontmatter(text: str) -> dict[str, str]:
    match = re.match(r"\A---\s*\n(.*?)\n---(?:\s*\n|\Z)", text, re.S)
    if not match:
        return {}
    values: dict[str, str] = {}
    for line in match.group(1).splitlines():
        item = re.match(r"^([a-z_]+):\s*(.*?)\s*$", line)
        if item:
            values[item.group(1)] = item.group(2).split(" #", 1)[0].strip()
    return values


def _dependencies(value: str) -> tuple[str, ...]:
    if not value:
        return ()
    inside = value.strip()
    if inside.startswith("[") and inside.endswith("]"):
        inside = inside[1:-1]
    return tuple(re.findall(r"T-[A-Za-z0-9]+", inside))


def _section(text: str, title: str) -> str:
    match = re.search(
        rf"^##\s+{re.escape(title)}\s*\n(.*?)(?=^##\s+|\Z)",
        text,
        re.M | re.S,
    )
    return match.group(1) if match else ""


def _section_headings(text: str) -> set[str]:
    return {match.strip() for match in re.findall(r"^##\s+(.+?)\s*$", text, re.M)}


def _files_rows(section: str) -> int:
    count = 0
    for line in section.splitlines():
        if not line.startswith("|") or re.match(r"\|\s*[-:]+", line):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if cells and cells[0].lower() not in {"path", "file", "files"}:
            count += 1
    return count


def _acceptance_criteria_count(section: str) -> int:
    return sum(1 for line in section.splitlines() if re.match(r"^\s*[-*]\s*AC\d+\b", line))


def _test_names(text: str) -> set[str]:
    tests = _section(text, "Tests to write")
    names: set[str] = set()
    if not tests:
        return names
    for line in tests.splitlines():
        if not line.startswith("|") or re.match(r"\|\s*[-:]+", line):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) < 2 or cells[0].lower() in {"file", "path"}:
            continue
        value = cells[1]
        for first, last in re.findall(r"\bAC\s*(\d+)\s*[–-]\s*AC?\s*(\d+)\b", value):
            names.update(f"AC{i}" for i in range(int(first), int(last) + 1))
        if re.search(r"\bAC(?:s| names?)\s+(?:names?\s+)?above\b|\bAC names above\b", value, re.I):
            names.update(_ac_test_names(text))
        for token in re.findall(r"`([^`]+)`", value):
            if token.startswith("AC"):
                continue
            # A test cell may say `file.ts: test_name`; that is one named test.
            token = token.rsplit(":", 1)[-1].strip()
            if token and not re.fullmatch(r"[A-Za-z0-9_./{}*-]+", token):
                names.add(token)
            elif token and "/" not in token and "." not in token:
                names.add(token)
    return names


def _ac_test_names(text: str) -> set[str]:
    names: set[str] = set()
    section = _section(text, "Acceptance criteria")
    for line in section.splitlines():
        if not re.match(r"^\s*[-*]\s*AC\d+\b", line) or "→" not in line:
            continue
        rhs = line.rsplit("→", 1)[1]
        # The last arrow maps the criterion to its test. Earlier arrows can
        # describe transitions within the scenario itself.
        match = re.match(r"\s*`([^`]+)`", rhs)
        if match:
            value = match.group(1).strip()
            if "::" not in value and ": " in value:
                value = value.rsplit(": ", 1)[-1].strip()
            for name in re.split(r"\s*\+\s*|\s*,\s*", value):
                if name:
                    names.add(name.rsplit("::", 1)[-1])
    return names


def _load_tasks(root: Path) -> list[Task]:
    tasks: list[Task] = []
    for path in sorted((root / "tasks").glob("*/T-*.md")):
        text = path.read_text(encoding="utf-8")
        values = _frontmatter(text)
        task_id = values.get("id", "")
        status = values.get("status", "")
        tasks.append(Task(path, task_id, status, _dependencies(values.get("depends_on", "")), text))
    return tasks


def validate(root: Path) -> list[str]:
    tasks = _load_tasks(root)
    errors: list[str] = []
    known_ids = {task.task_id for task in tasks}
    for task in tasks:
        relative = task.path.relative_to(root).as_posix()
        if task.status == "blocked":
            errors.append(f"{relative}: status blocked is not allowed")
        for dependency in task.dependencies:
            if dependency not in known_ids:
                errors.append(f"{relative}: unknown dependency {dependency}")
        if task.status not in ACTIVE_STATUSES:
            continue
        headings = _section_headings(task.text)
        for heading in REQUIRED_SECTIONS:
            if heading not in headings:
                errors.append(f"{relative}: missing required section ## {heading}")
        if SECURITY_TRIGGER.search(task.text.split("## Report", 1)[0]) and "Security" not in headings:
            errors.append(f"{relative}: security trigger without ## Security")
        for pattern, description in REPLACED_STACK:
            if pattern.search(task.text.split("## Report", 1)[0]):
                errors.append(f"{relative}: replaced dependency {description}")
        file_count = _files_rows(_section(task.text, "Files"))
        ac_count = _acceptance_criteria_count(_section(task.text, "Acceptance criteria"))
        test_count = len(_test_names(task.text))
        if file_count > 8:
            errors.append(f"{relative}: {file_count} Files rows exceeds 8")
        if ac_count > 8:
            errors.append(f"{relative}: {ac_count} acceptance criteria exceeds 8")
        if test_count > 12:
            errors.append(f"{relative}: {test_count} test names exceeds 12")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args(argv)
    errors = validate(args.root.resolve())
    if errors:
        print("Task lint failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Task files pass hygiene checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

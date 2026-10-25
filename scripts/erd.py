#!/usr/bin/env python3
"""ERD generator and drift check for spec/29-erd.md (ERD-01..ERD-05).

The normative DDL lives in spec/06-domain-model.md §3-§4 and spec/04-policy-engine.md §2.
This script parses it and owns every block of spec/29-erd.md between
`<!-- BEGIN GENERATED: <name> -->` and `<!-- END GENERATED: <name> -->`.

Usage:
  scripts/erd.py --write        regenerate the generated blocks of spec/29-erd.md (architect only)
  scripts/erd.py --check        fail if spec/29-erd.md is stale against the DDL (CI)
  scripts/erd.py --migrations   fail if migrations/*.sql drift from the normative DDL (CI, every task)

No dependencies beyond the Python 3 standard library.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DDL_SOURCES = [ROOT / "spec/06-domain-model.md", ROOT / "spec/04-policy-engine.md"]
ERD_DOC = ROOT / "spec/29-erd.md"
MIGRATIONS = ROOT / "migrations"

MODULES = ["core", "identity", "scheduling", "enrollment", "billing",
           "attendance", "rental", "notify", "audit", "feedback", "platform"]

# spec/06 section heading -> owner module
SECTION_MODULE = {
    "3": "core", "4.1": "core", "4.2": "identity", "4.3": "scheduling",
    "4.4": "enrollment", "4.5": "billing", "4.6": "attendance", "4.7": "rental",
    "4.8": "notify", "4.9": "audit", "4.10": "feedback",
}
TABLE_MODULE_OVERRIDE = {"policies": "core"}  # spec/04 §2

# ERD-03: tables without row-level security (spec/06 §4.1 note, §4.11, comments).
NO_RLS = {
    "tenants": "read through SECURITY DEFINER resolve_tenant(slug)",
    "outbox": "system table, dispatcher only",
    "event_receipts": "system table, dispatcher only",
    "jobs": "system table, runner only",
    "tenant_keys": "accessed only by core::crypto",
    "audit.checkpoints": "spec/06 §4.11",
    "cron_state": "system table, cron planner only",
    "platform.accounts": "platform schema, role eq_platform only",
    "platform.account_tenants": "platform schema, role eq_platform only",
    "platform.plans": "platform schema, role eq_platform only",
    "platform.subscriptions": "platform schema, role eq_platform only",
    "platform.usage_snapshots": "platform schema, role eq_platform only",
}

# ERD-02: logical references. Columns that point at another table WITHOUT a database FK,
# because the owner is another module (ARC: modules don't share tables) or the target is polymorphic.
LOGICAL_REFS = {
    ("calendar_holds", "subject_id"): ("resources | persons", "polymorphic by subject_kind"),
    ("calendar_holds", "owner_id"): ("sessions | reservations", "polymorphic by owner_kind"),
    ("series_members", "enrollment_id"): ("enrollments", "set by enrollment module (event)"),
    ("session_participants", "enrollment_id"): ("enrollments", "set by enrollment module (event)"),
    ("invoice_lines", "reservation_id"): ("reservations", "rental is optional (MOD-04)"),
    ("reservations", "invoice_id"): ("invoices", "billing is optional (MOD-04)"),
    ("persons", "avatar_file_id"): ("files", "set after upload"),
    ("outbox", "aggregate_id"): ("any aggregate", "event source"),
    ("audit.entries", "entity_id"): ("any aggregate", "by entity_type"),
    ("audit.entries", "actor_id"): ("persons", "NULL = system; no FK so audit survives person deletion"),
    ("attention_items", "ref_id"): ("any aggregate", "by ref_type"),
    ("event_receipts", "event_id"): ("outbox", "dedupe of handled events"),
}

# ERD-04: data class per column (SBD-10). Default is S1 Internal.
S3 = {
    ("credentials", "password_hash"), ("credentials", "totp_secret_enc"),
    ("refresh_sessions", "token_hash"), ("invites", "token_hash"),
    ("password_resets", "token_hash"), ("channel_configs", "config_enc"),
    ("tenant_keys", "dek_enc"), ("totp_recovery_codes", "code_hash"),
}
S2 = {
    ("persons", "legal_name_enc"), ("persons", "email_enc"), ("persons", "email_bidx"),
    ("persons", "phone_enc"), ("persons", "phone_bidx"), ("persons", "birth_date_enc"),
    ("persons", "address_enc"), ("registrations", "applicant_enc"),
    ("refresh_sessions", "ip"), ("refresh_sessions", "user_agent"), ("audit.entries", "ip"),
    ("checkins", "gps_lat"), ("checkins", "gps_lng"), ("checkins", "gps_accuracy_m"),
    ("checkins", "gps_distance_m"), ("checkins", "face_ratio_bps"),
    ("platform.accounts", "email_enc"), ("platform.accounts", "email_bidx"),
}
S0 = {
    ("tenants", "slug"), ("tenants", "name"),
    ("offerings", "key"), ("offerings", "name"), ("offerings", "description"),
    ("platform.plans", "id"), ("platform.plans", "name"), ("platform.plans", "limits"), ("platform.plans", "price"),
}
# ERD-05: plaintext free-text columns that may describe a minor (SBD-10 says S2). Open question.
CLASS_GAPS = {
    ("sessions", "notes"), ("leaves", "reason"), ("attendance_marks", "note"),
    ("progress_notes", "ratings"), ("progress_notes", "comment"),
    ("instructor_ratings", "comment"), ("registrations", "review_note"),
    ("checkins", "review_note"), ("deletion_requests", "reason"), ("enrollment_changes", "reason"),
}

CONSTRAINT_WORDS = {"PRIMARY", "UNIQUE", "CHECK", "CONSTRAINT", "EXCLUDE", "FOREIGN"}
TYPE_STOP = {"NOT", "NULL", "PRIMARY", "REFERENCES", "DEFAULT", "CHECK", "UNIQUE",
             "GENERATED", "CONSTRAINT", "COLLATE"}


def split_top(text, sep=","):
    parts, depth, cur = [], 0, []
    for ch in text:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == sep and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return parts


def matching_paren(text, start):
    depth = 0
    for i in range(start, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i
    raise ValueError("unbalanced parentheses")


def parse_column(item, comment):
    toks = item.split()
    name = toks[0].strip('"')
    type_toks = []
    for t in toks[1:]:
        if t.upper() in TYPE_STOP:
            break
        type_toks.append(t)
    rest = " ".join(toks[1 + len(type_toks):])
    up = rest.upper()
    ref = re.search(r"REFERENCES\s+([\w.]+)", rest, re.I)
    return {
        "name": name,
        "type": " ".join(type_toks).lower(),
        "not_null": "NOT NULL" in up or "PRIMARY KEY" in up,
        "pk": "PRIMARY KEY" in up,
        "unique": bool(re.search(r"\bUNIQUE\b", up)),
        "fk": ref.group(1) if ref else None,
        "comment": comment.strip(),
    }


def parse_create(body):
    """Parse the text inside CREATE TABLE (...). Comments are carried to the column they follow."""
    marked = re.sub(r"--([^\n]*)", lambda m: "\x00" + m.group(1).replace(",", "\x01") + "\x02", body)
    cols, table_pk, table_unique, note = [], [], [], ""
    pending = None
    for raw in split_top(marked):
        comments = re.findall(r"\x00(.*?)\x02", raw)
        item = re.sub(r"\x00.*?\x02", "", raw).strip()
        lead = raw.lstrip()
        # a comment at the start of an item belongs to the previous column
        if comments and lead.startswith("\x00"):
            text = comments.pop(0).replace("\x01", ",").strip()
            if pending is None:
                note = text  # a comment on the CREATE TABLE line describes the table
            else:
                pending["comment"] = (pending["comment"] + " " + text).strip()
        if not item:
            continue
        first = item.split()[0].upper()
        if first in CONSTRAINT_WORDS:
            m = re.match(r"PRIMARY\s+KEY\s*\(([^)]*)\)", item, re.I)
            if m:
                table_pk = [c.strip() for c in m.group(1).split(",")]
            m = re.match(r"UNIQUE\s*\(([^)]*)\)", item, re.I)
            if m:
                table_unique.append([c.strip() for c in m.group(1).split(",")])
            continue
        col = parse_column(item, " ".join(c.replace("\x01", ",") for c in comments))
        cols.append(col)
        pending = col
    for c in cols:
        if c["name"] in table_pk:
            c["pk"] = True
            c["not_null"] = True
        for u in table_unique:
            if u == [c["name"]]:
                c["unique"] = True
    return cols, note


def parse_sql(sql, section=None, tables=None, order=None):
    """Parse CREATE TABLE and ALTER TABLE ... ADD COLUMN statements from SQL text."""
    tables = {} if tables is None else tables
    order = [] if order is None else order
    pos = 0
    pattern = re.compile(r"CREATE TABLE\s+(?:IF NOT EXISTS\s+)?([\w.]+)\s*\(|ALTER TABLE\s+([\w.]+)\s+(ADD COLUMN[^;]*);", re.I)
    for m in pattern.finditer(sql):
        if m.start() < pos:
            continue
        if m.group(1):
            name = m.group(1)
            open_i = m.end() - 1
            close_i = matching_paren(sql, open_i)
            pos = close_i
            preceding = sql[max(0, sql.rfind("\n\n", 0, m.start())):m.start()]
            module = section(name, preceding) if section else None
            cols, note = parse_create(sql[open_i + 1:close_i])
            tables[name] = {"module": module, "cols": cols, "note": note}
            order.append(name)
        else:
            name = m.group(2)
            if name not in tables or name.startswith("<"):
                continue
            for part in split_top(m.group(3)):
                part = re.sub(r"^\s*ADD COLUMN\s+", "", part.strip(), flags=re.I)
                if part:
                    col = parse_column(part, "")
                    col["added"] = True
                    tables[name]["cols"].append(col)
    return tables, order


def load_spec():
    tables, order = {}, []
    for src in DDL_SOURCES:
        text = src.read_text()
        heading_iter = list(re.finditer(r"^##+ (\d+(?:\.\d+)?)\b.*$", text, re.M))
        for block in re.finditer(r"```sql\n(.*?)```", text, re.S):
            sec = None
            for h in heading_iter:
                if h.start() < block.start():
                    sec = h.group(1)

            def section(name, preceding, sec=sec):
                if name in TABLE_MODULE_OVERRIDE:
                    return TABLE_MODULE_OVERRIDE[name]
                if sec in ("4.11", "4.12"):
                    m = re.findall(r"^-- (\w+)", preceding, re.M)
                    if m and m[-1] in MODULES:
                        return m[-1]
                    return "audit" if name.startswith("audit.") else None
                return SECTION_MODULE.get(sec)
            parse_sql(block.group(1), section, tables, order)
    for name in order:
        if tables[name]["module"] is None:
            sys.exit(f"erd.py: no owner module for table {name}")
    return tables, order


def data_class(table, col):
    key = (table, col)
    if key in S3:
        return "S3"
    if key in S2:
        return "S2"
    if key in CLASS_GAPS:
        return "S1 ⚠"
    if key in S0:
        return "S0"
    return "S1"


def mm_name(table):
    return table.replace(".", "_")


def mm_type(t):
    t = t.replace("double precision", "float8").replace(" ", "_")
    return re.sub(r"[^A-Za-z0-9_()\[\]-]", "-", t) or "unknown"


def mm_comment(s):
    return s.replace('"', "'")


def relationships(tables, names, include_logical=True):
    """FK edges (solid) and logical references (dotted). Edges to `tenants` are omitted:
    every tenant-owned table has one (DOM-01), and drawing them hides the real structure."""
    rels = []
    for t in names:
        single_pk = sum(1 for x in tables[t]["cols"] if x["pk"]) == 1
        for c in tables[t]["cols"]:
            target = c["fk"]
            if target and target != "tenants" and target in tables:
                child_card = "o|" if c["unique"] or (c["pk"] and single_pk) else "o{"
                parent_card = "||" if c["not_null"] else "|o"
                rels.append(f'  {mm_name(target)} {parent_card}--{child_card} {mm_name(t)} : "{c["name"]}"')
            if include_logical and (t, c["name"]) in LOGICAL_REFS:
                tgt = LOGICAL_REFS[(t, c["name"])][0]
                for one in (x.strip() for x in tgt.split("|")):
                    if one in tables:
                        rels.append(f'  {mm_name(one)} |o..o{{ {mm_name(t)} : "{c["name"]} (logical)"')
    return rels


def gen_overview(tables, order):
    # Every entity is declared (with its owner module as the only attribute) so tables without edges still show.
    out = ["```mermaid", "erDiagram"]
    for mod in MODULES:
        names = [t for t in order if tables[t]["module"] == mod]
        if names:
            out.append(f"  %% {mod}")
            for t in names:
                out.append(f"  {mm_name(t)} {{\n    text module \"{mod}\"\n  }}")
    out.extend(relationships(tables, order))
    out.append("```")
    return "\n".join(out)


def gen_module(tables, order, mod):
    names = [t for t in order if tables[t]["module"] == mod]
    out = ["```mermaid", "erDiagram"]
    for t in names:
        out.append(f"  {mm_name(t)} {{")
        for c in tables[t]["cols"]:
            keys = [k for k, on in (("PK", c["pk"]), ("FK", bool(c["fk"])), ("UK", c["unique"] and not c["pk"])) if on]
            line = f"    {mm_type(c['type'])} {c['name']}"
            if keys:
                line += " " + ", ".join(keys)
            cls = data_class(t, c["name"])
            if cls != "S1":
                line += f' "{mm_comment(cls)}"'
            out.append(line)
        out.append("  }")
    rels = relationships(tables, names)
    # incoming FKs from other modules are shown in those modules; external parents appear by name only
    out.extend(rels)
    out.append("```")
    return "\n".join(out)


def gen_dictionary(tables, order, mod):
    out = []
    for t in [x for x in order if tables[x]["module"] == mod]:
        rls = "no — " + NO_RLS[t] if t in NO_RLS else ("yes" if any(c["name"] == "tenant_id" for c in tables[t]["cols"]) else "no — global table")
        note = f" · {tables[t]['note']}" if tables[t]["note"] else ""
        out.append(f"#### `{t}`\n\nOwner: `{mod}` · RLS: {rls}{note}\n")
        out.append("| Column | Type | Null | Key | Class | References | Notes |")
        out.append("|---|---|---|---|---|---|---|")
        for c in tables[t]["cols"]:
            keys = "/".join(k for k, on in (("PK", c["pk"]), ("FK", bool(c["fk"])), ("UK", c["unique"] and not c["pk"])) if on)
            ref = f"`{c['fk']}`" if c["fk"] else ""
            if (t, c["name"]) in LOGICAL_REFS:
                tgt, why = LOGICAL_REFS[(t, c["name"])]
                ref = f"`{tgt}` (logical: {why})"
            note = c["comment"].replace("|", "\\|")
            if c.get("added"):
                note = (note + " (added by ALTER, §4.11)").strip()
            out.append(f"| `{c['name']}` | `{c['type']}` | {'' if c['not_null'] else 'yes'} | {keys} | {data_class(t, c['name'])} | {ref} | {note} |")
        out.append("")
    return "\n".join(out).rstrip() + "\n"


def gen_inventory(tables, order):
    out = ["| Module | Tables | Count |", "|---|---|---|"]
    for mod in MODULES:
        names = [t for t in order if tables[t]["module"] == mod]
        out.append(f"| `{mod}` | {', '.join('`' + n + '`' for n in names)} | {len(names)} |")
    out.append(f"| **total** | | **{len(order)}** |")
    return "\n".join(out)


def gen_logical(tables):
    out = ["| Table.column | Points at | Why there is no FK |", "|---|---|---|"]
    for (t, c), (tgt, why) in LOGICAL_REFS.items():
        out.append(f"| `{t}.{c}` | `{tgt}` | {why} |")
    return "\n".join(out)


def gen_classes():
    out = ["| Class | Columns |", "|---|---|"]
    for name, cols in (("S3 Secret", S3), ("S2 Sensitive PII", S2), ("S0 Public", S0)):
        out.append(f"| {name} | {', '.join('`' + t + '.' + c + '`' for t, c in sorted(cols))} |")
    out.append("| S1 Internal | every other column |")
    out.append(f"| ⚠ open (ERD-05) | {', '.join('`' + t + '.' + c + '`' for t, c in sorted(CLASS_GAPS))} |")
    return "\n".join(out)


def generated_blocks():
    tables, order = load_spec()
    blocks = {
        "inventory": gen_inventory(tables, order),
        "overview": gen_overview(tables, order),
        "logical-refs": gen_logical(tables),
        "classes": gen_classes(),
    }
    for mod in MODULES:
        blocks[f"diagram-{mod}"] = gen_module(tables, order, mod)
        blocks[f"dictionary-{mod}"] = gen_dictionary(tables, order, mod)
    return blocks


def render(doc, blocks):
    def repl(m):
        name = m.group(1)
        if name not in blocks:
            sys.exit(f"erd.py: unknown generated block {name}")
        return f"<!-- BEGIN GENERATED: {name} -->\n{blocks[name].rstrip()}\n<!-- END GENERATED: {name} -->"
    new = re.sub(r"<!-- BEGIN GENERATED: ([\w-]+) -->.*?<!-- END GENERATED: \1 -->", repl, doc, flags=re.S)
    missing = [n for n in blocks if f"BEGIN GENERATED: {n} -->" not in new]
    if missing:
        sys.exit(f"erd.py: spec/29-erd.md lacks generated blocks: {', '.join(missing)}")
    return new


TYPE_ALIASES = {
    "integer": "int", "int4": "int", "int8": "bigint", "bool": "boolean", "float8": "double precision",
    "float4": "real", "timestamp with time zone": "timestamptz", "character": "char",
    "serial8": "bigserial", "varchar": "character varying",
}


def norm_type(t):
    t = re.sub(r"\s+", " ", t.lower().strip())
    t = re.sub(r"^character\(", "char(", t)
    return TYPE_ALIASES.get(t, t)


def check_migrations():
    spec, _ = load_spec()
    files = sorted(MIGRATIONS.glob("*.sql")) if MIGRATIONS.is_dir() else []
    mig, _ = {}, None
    for f in files:
        parse_sql(re.sub(r"--[^\n]*", "", f.read_text()), None, mig, [])
    errors = []
    for t, info in mig.items():
        if t not in spec:
            errors.append(f"{t}: table not in the normative DDL (spec/06 §4)")
            continue
        scols = {c["name"]: c for c in spec[t]["cols"]}
        mcols = {c["name"]: c for c in info["cols"]}
        for name in scols.keys() - mcols.keys():
            errors.append(f"{t}.{name}: missing in migrations")
        for name in mcols.keys() - scols.keys():
            errors.append(f"{t}.{name}: not in the normative DDL")
        for name in scols.keys() & mcols.keys():
            s, m = scols[name], mcols[name]
            if norm_type(s["type"]) != norm_type(m["type"]):
                errors.append(f"{t}.{name}: type {m['type']!r} != spec {s['type']!r}")
            if s["not_null"] != m["not_null"]:
                errors.append(f"{t}.{name}: nullability differs from spec")
            if (s["fk"] or None) != (m["fk"] or None):
                errors.append(f"{t}.{name}: references {m['fk']} != spec {s['fk']}")
    pending = sorted(set(spec) - set(mig))
    print(f"erd.py: {len(mig)} migrated tables checked against spec, {len(pending)} not migrated yet")
    if errors:
        print("Schema drift (ERD-03, DOM-01: migrations MUST produce exactly the spec/06 schema):", file=sys.stderr)
        for e in errors:
            print("  " + e, file=sys.stderr)
        return 1
    return 0


def main(argv):
    if argv == ["--migrations"]:
        return check_migrations()
    if argv not in (["--write"], ["--check"]):
        print(__doc__, file=sys.stderr)
        return 2
    doc = ERD_DOC.read_text()
    new = render(doc, generated_blocks())
    if argv == ["--write"]:
        ERD_DOC.write_text(new)
        print("erd.py: spec/29-erd.md regenerated")
        return 0
    if new != doc:
        print("erd.py: spec/29-erd.md is stale; the architect runs scripts/erd.py --write", file=sys.stderr)
        return 1
    print("erd.py: spec/29-erd.md matches the DDL")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="${1:-$(cd -- "${script_dir}/.." && pwd)}"
if [[ $# -gt 1 ]]; then
  printf 'usage: %s [repository-root]\n' "$0" >&2
  exit 2
fi

python3 - "$root" "$script_dir/check-literals.allow" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1]).resolve()
allow_path = Path(sys.argv[2])
if not (root / "crates").is_dir():
    print(f"not a repository root: {root}", file=sys.stderr)
    sys.exit(2)

currency_codes = set("""
AED AFN ALL AMD ANG AOA ARS AUD AWG AZN BAM BBD BDT BGN BHD BIF BMD BND BOB BOV BRL BSD BTN BWP BYN BZD
CAD CDF CHE CHF CHW CLF CLP CNY COP COU CRC CUP CVE CZK DJF DKK DOP DZD EGP ERN ETB EUR FJD FKP GBP GEL
GHS GIP GMD GNF GTQ GYD HKD HNL HTG HUF IDR ILS INR IQD IRR ISK JMD
KWD KYD KZT LAK LBP LKR LRD LSL LYD MAD MDL MGA MKD MMK MNT MOP MRU MUR MVR MWK MXN MXV MYR MZN NAD
NGN NIO NOK NPR NZD OMR PAB PEN PGK PHP PKR PLN PYG QAR RON RSD RUB RWF SAR SBD SCR SDG SEK SGD SHP
SLE SLL SOS SRD SSP STN SVC SYP SZL THB TJS TMT TND TOP TRY TTD TWD TZS UAH UGX USD USN UYI UYU UYW
UZS VED VES VND VUV WST XAF XAG XAU XBA XBB XBC XBD XCD XCG XDR XOF XPD XPF XPT XSU XTS XUA XXX YER
ZAR ZMW ZWL
""".split())
currency_codes.add("J" + "OD")
currency_codes.update("""
JPY KES KGS KHR KMF KPW KRW
""".split())
allow_rules = []
if allow_path.exists():
    for line_number, raw_line in enumerate(allow_path.read_text(encoding="utf-8").splitlines(), 1):
        rule, marker, justification = raw_line.partition("#")
        rule = rule.strip()
        if not rule:
            continue
        if not marker or not justification.strip():
            print(f"{allow_path}:{line_number}: allow-list entries need a justification comment", file=sys.stderr)
            sys.exit(2)
        filename, separator, line = rule.rpartition(":")
        if not separator or not filename or (line != "*" and not line.isdigit()):
            print(f"{allow_path}:{line_number}: expected path:line or path:*", file=sys.stderr)
            sys.exit(2)
        if Path(filename).is_absolute() or ".." in Path(filename).parts:
            print(f"{allow_path}:{line_number}: allow-list paths must stay inside the repository", file=sys.stderr)
            sys.exit(2)
        allow_rules.append((filename, line))

def line_number(text, offset):
    return text.count("\n", 0, offset) + 1

def mask_span(chars, start, end):
    for position in range(start, end):
        if chars[position] != "\n":
            chars[position] = " "

def mask_and_strings(text):
    chars = list(text)
    strings = []
    position = 0
    length = len(text)
    while position < length:
        if text.startswith("//", position):
            end = text.find("\n", position)
            if end == -1:
                end = length
            mask_span(chars, position, end)
            position = end
            continue
        if text.startswith("/*", position):
            start = position
            depth = 1
            position += 2
            while position < length and depth:
                if text.startswith("/*", position):
                    depth += 1
                    position += 2
                elif text.startswith("*/", position):
                    depth -= 1
                    position += 2
                else:
                    position += 1
            mask_span(chars, start, position)
            continue

        raw_prefix = re.match(r"(?:br|r)(#+)?\"", text[position:])
        if raw_prefix:
            start = position
            hashes = raw_prefix.group(1) or ""
            content_start = position + len(raw_prefix.group(0))
            terminator = '"' + hashes
            end = text.find(terminator, content_start)
            if end == -1:
                end = length
                content_end = end
            else:
                content_end = end
                end += len(terminator)
            strings.append((line_number(text, content_start), text[content_start:content_end]))
            mask_span(chars, start, end)
            position = end
            continue

        if text.startswith('b"', position):
            start = position
            quote = position + 1
        elif text[position] == '"':
            start = position
            quote = position
        else:
            quote = -1
            start = position
        if quote >= 0:
            content_start = quote + 1
            cursor = content_start
            while cursor < length:
                if text[cursor] == "\\":
                    cursor += 2
                elif text[cursor] == '"':
                    break
                else:
                    cursor += 1
            content_end = min(cursor, length)
            strings.append((line_number(text, content_start), text[content_start:content_end]))
            end = min(cursor + 1, length)
            mask_span(chars, start, end)
            position = end
            continue

        if text[position] == "'":
            cursor = position + 1
            if cursor < length and text[cursor] != "\n":
                cursor += 2 if text[cursor] == "\\" else 1
                if cursor < length and text[cursor] == "'":
                    mask_span(chars, position, cursor + 1)
                    position = cursor + 1
                    continue
        position += 1
    return "".join(chars), strings

def in_marked_const(code, raw_lines, offset):
    lines = code.splitlines(keepends=True)
    start = 0
    for index, line in enumerate(lines):
        end = start + len(line)
        if re.match(r"\s*(?:(?:pub(?:\([^)]*\))?)\s+)?const\b", line):
            previous = raw_lines[index - 1] if index else ""
            if re.search(r"//\s*technical:", previous):
                semicolon = code.find(";", start)
                const_end = end if semicolon == -1 else semicolon + 1
                if start <= offset < const_end:
                    return True
        start = end
    return False

issues = []
for path in sorted((root / "crates").glob("*/src/**/*.rs")):
    relative = path.relative_to(root)
    parts = relative.parts
    source_index = parts.index("src")
    source_parts = parts[source_index + 1 :]
    basename = path.name
    in_domain_tree = "domain" in source_parts[:-1]
    named_domain_or_service = basename.startswith("domain") or basename.startswith("service")
    if not (in_domain_tree or named_domain_or_service):
        continue

    text = path.read_text(encoding="utf-8")
    code, strings = mask_and_strings(text)
    for line, value in strings:
        if re.search(r"\b(?:Asia|America|Europe)/[A-Za-z0-9_+./-]+", value):
            issues.append((str(relative), line, "IANA timezone name in string literal"))
        if re.search(r"\bWIB\b", value):
            issues.append((str(relative), line, "timezone abbreviation WIB in string literal"))
        for match in re.finditer(r"(?<![A-Z])[A-Z]{3}(?![A-Z])", value):
            if match.group(0) in currency_codes:
                issues.append((str(relative), line, f"currency code {match.group(0)} in string literal"))

    raw_lines = text.splitlines()
    for match in re.finditer(r"(?<![A-Za-z0-9_.])(?:0[xX][0-9A-Fa-f_]+|0[oO][0-7_]+|0[bB][01_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)?)(?![A-Za-z0-9_.])", code):
        token = match.group(0)
        line = line_number(text, match.start())
        if "." in token:
            fraction = token.split(".", 1)[1].split("e", 1)[0].split("E", 1)[0]
            if len(fraction.replace("_", "")) >= 3:
                issues.append((str(relative), line, f"floating-point literal {token} has at least 3 decimal places"))
        else:
            if in_marked_const(code, raw_lines, match.start()):
                continue
            if token.lower().startswith("0x"):
                numeric_value = int(token[2:].replace("_", ""), 16)
            elif token.lower().startswith("0o"):
                numeric_value = int(token[2:].replace("_", ""), 8)
            elif token.lower().startswith("0b"):
                numeric_value = int(token[2:].replace("_", ""), 2)
            else:
                numeric_value = int(token.replace("_", ""))
            if numeric_value >= 1000:
                issues.append((str(relative), line, f"integer literal {token} is >= 1000; use a // technical: const or policy field"))

    for line, source_line in enumerate(code.splitlines(), 1):
        if re.search(r"\.day\s*\(\s*\)\s*==\s*\d+", source_line):
            issues.append((str(relative), line, "calendar day comparison uses a numeric literal"))

def is_allowed(filename, line):
    return any(path == filename and (allowed_line == "*" or int(allowed_line) == line) for path, allowed_line in allow_rules)

for filename, line, reason in sorted(set(issues)):
    if is_allowed(filename, line):
        continue
    print(f"{filename}:{line}: {reason}")
sys.exit(1 if any(not is_allowed(filename, line) for filename, line, _ in issues) else 0)
PY

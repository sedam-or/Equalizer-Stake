# Security Policy

## Supported versions

| Version | Supported |
|---|---|
| Latest minor of the current major | ✅ |
| Previous minor | ✅ security fixes for 90 days after a new minor |
| `legacy/` (v0 capstone) | ❌ archived, not deployed |

## Reporting a vulnerability

**Please don't open public issues for security problems.**

Report privately through **GitHub Security Advisories** ("Report a vulnerability" on the Security tab). Include affected versions, reproduction steps, impact, and any suggested fix.

What to expect:
- Acknowledgement within **48 hours**.
- A triage decision and severity (CVSS 3.1) within **5 days**.
- A fix or mitigation plan within **30 days** for high/critical issues.
- Coordinated disclosure: we publish an advisory and a CVE once a fix is released, at most **90 days** after the report unless we agree otherwise. We credit you if you wish.

## Scope

In scope: the `equalizer` server, the web app, the desktop app, the official deploy templates and install script, and the managed cloud.

Out of scope: denial of service by volume, social engineering, issues in third-party providers (SMTP or WhatsApp vendors), and self-hosted instances misconfigured against the docs.

Please don't access data that isn't yours, don't degrade service, and stop and report as soon as you find a problem.

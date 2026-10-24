# ADR-0014: Adopt open-source building blocks; build only the differentiators

- Status: Proposed (2026-10-25), accepted on merge of the PR that adds `spec/20`–`spec/24`

## Context
Ritma is a small open-source project implemented largely by agents (Codex) and contributors with mixed experience. Generic infrastructure written from scratch (validation, auth primitives, file handling, forms, CI) is where most security bugs and most maintenance cost come from. Mature, permissively licensed projects already solve these problems, and contributors already know them.

At the same time, three parts of Ritma are the product itself and have no off-the-shelf equivalent that fits: the conflict ledger (DOM-10), the typed effective-dated policy engine (ADR-0004), and the course-school domain rules.

Until now, library choices were spread across 02 §11, 08 §1 and individual tasks, and some generic pieces (validation rules, client schemas) had no named library at all, which invited hand-written duplicates on the server and in the browser.

## Decision
- **Adopt first.** Every generic concern maps to one named open-source project in `spec/24-open-source-stack.md` (OSS-\*). Tasks use that project; new generic code needs a reason recorded there.
- **One seam per security-relevant library** (`eq_core::crypto`, `outbound`, `files`, `fields`; `web/src/lib/*`), so a library can be replaced in one place (SBD-07).
- **One definition per validation rule.** Field kinds in `eq_core::fields` drive the server parser, the OpenAPI schema, and the generated client zod schemas (spec 22 VAL-20/30). Parity is proven by shared test cases (VAL-31).
- **Build only:** the conflict ledger and alternatives, the policy engine, the outbox/job queue over `sqlx`, `Money`, the audit chain, the `PolicyForm` renderer and the field-kind glue (OSS §5), each with the evaluated alternatives written down.
- **No extra runtime services** (OSS-05): libraries over servers, PostgreSQL stays the only stateful dependency (ADR-0001/0002).
- **Supply chain as code:** license gate, SHA-pinned actions, Dependabot, CodeQL, Scorecard, gitleaks, SBOM, cosign, provenance (OSS §4/§6).

## Consequences
- ✅ Less code to write and review; well-known APIs for contributors; security fixes arrive through dependency updates.
- ✅ Server and client validation can't drift silently, because both are generated from one rule set.
- ✅ CI and release security are visible and checked from the first PR.
- ❌ More dependencies to watch. Mitigated by `cargo deny`, `pnpm audit`, OSV-Scanner, Dependabot and the CQ-27 review.
- ❌ Some libraries are young (`@hey-api/openapi-ts` is pre-1.0). Mitigated by keeping them behind generation steps whose output is committed and diffed (TST-10), so a replacement changes generated files, not hand-written code.

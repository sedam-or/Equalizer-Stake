# Governance

## Today: maintainer-led

The project is led by its founding maintainer (BDFL model), who has final say on scope, architecture and releases. This keeps decisions fast while the project is young.

## Growing up: maintainers council

When there are **3 or more active maintainers** (each with ≥ 10 merged non-trivial PRs and 6 months of activity), governance moves to a **maintainers council**:
- Decisions by lazy consensus on the PR or RFC; if contested, a simple majority vote of the council.
- The founding maintainer keeps a tie-break vote for 12 months after the transition, then becomes a regular council member.
- New maintainers are nominated by any maintainer and approved by the council.

## What needs an RFC

Open a PR adding `rfcs/NNNN-title.md` (copy `rfcs/0000-template.md`) for:
- a new module or a new `ModuleId`;
- a breaking change to a policy schema (bumping `SCHEMA_VERSION`);
- a breaking API change (`/api/v2`);
- a change to licensing, governance, telemetry or data handling.

RFCs stay open for comments for at least **7 days**, are labelled `rfc`, and are accepted or declined with a written rationale.

## Roles

| Role | Can |
|---|---|
| Contributor | Open issues and PRs, take part in discussions and RFCs |
| Triager | Label, close duplicates, ask for repros |
| Maintainer | Review, merge, release, own areas in CODEOWNERS. Task reviews follow `WORKFLOW.md` §5 |

## Trademark

The code is AGPL-3.0, while the name and logo are trademarks of the project (see `TRADEMARK.md`). Forks are welcome and must use a different name.

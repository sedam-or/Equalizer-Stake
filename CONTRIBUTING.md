# Contributing — Equalizer-Stake

## Workflow

1. Baca `TASK_WORKFLOW.md` sebelum mulai task apapun
2. Baca `PROJECT_RULES.md` untuk constraint teknis
3. Baca SRS yang relevan di `docs/`
4. Ikuti 12-step workflow dari TASK_WORKFLOW.md
5. Tulis report di `tasks/report_tasks.md` setelah selesai

## Branch Strategy

- `main` — stable, production-ready
- `develop` — integration branch
- `feature/T-XXX-nama` — per task branch
- `hotfix/nama` — urgent fixes

## Commit Convention

```
[T-XXX] type: description

Types: feat, fix, refactor, test, docs, chore, security
Example: [T-003] feat: implement JWT authentication with RBAC
```

## Code Standards

- **Backend**: PEP 8, type hints, docstrings
- **Frontend**: ESLint, Prettier, TypeScript
- **Security**: No hardcoded secrets, PII encrypted, audit log immutable
- **Language**: Code in English, UI in Indonesian

## Pull Request

1. Branch dari `develop`
2. Pastikan semua test pass
3. Self-review dari sisi security (OWASP Top 10)
4. Self-review dari sisi performance (NFR targets)
5. Submit PR dengan description yang jelas
6. Reference FR/UC terkait dari SRS

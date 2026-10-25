# Releasing

## Versioning

- **SemVer** for the app: `MAJOR.MINOR.PATCH`. `/api/v1` is stable within major 1 (API-13).
- Each module's **policy schema version** (`SCHEMA_VERSION`) is versioned separately. A bump requires `migrate_policy` and a changelog entry under "Policy changes".
- DB migrations are forward-only (OPS-04).

## Process

1. Merge to `main` with commits that follow `COMMIT_CONVENTION.md` (types map to SemVer bumps and changelog sections, §2). **release-please** keeps a release PR with the changelog.
2. Merge the release PR, which creates tag `vX.Y.Z`.
3. `release.yml` (DST-05) builds the binaries, the multi-arch image, the desktop installers, the SBOMs, the checksums and the cosign signatures, then publishes the GitHub Release and `latest.json`.
4. A bot opens PRs bumping the image tag in `deploy/*` templates.
5. Announce: GitHub Discussions (Announcements), Discord/Telegram, and the update notice in the app.

## Checklist (release PR)

- [ ] CI green, including e2e and the backup round-trip (TST-13)
- [ ] Policy schema bumps listed, with migration tests
- [ ] Upgrade notes (manual steps should be none)
- [ ] Docs updated for user-visible changes (EN + ID)
- [ ] Translations ≥ 95% for `id`

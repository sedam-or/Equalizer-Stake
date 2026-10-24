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

## Maintenance and hotfix releases

- **Hotfix** (can't wait for the next release): `hotfix/v<X.Y.Z>-<slug>` PR to `main`, then merge the release-please PR immediately (`BRANCHING.md` §6). Security fixes are prepared privately through a GitHub Security Advisory first (SECURITY.md).
- **Patch for the previous minor** (from v1.0, 90 days, SECURITY.md): the fix lands on `main`, is cherry-picked to `release/vX.Y` through a `backport/…` PR, then a maintainer runs the `release-please` workflow manually with `target-branch: release/vX.Y`. The release must not move "latest" (GitHub release flag, image tag, `latest.json`); T-704 implements that guard (`BRANCHING.md` §5).

## Checklist (release PR)

- [ ] CI green, including e2e and the backup round-trip (TST-13)
- [ ] Policy schema bumps listed, with migration tests
- [ ] Upgrade notes (manual steps should be none)
- [ ] Docs updated for user-visible changes (EN + ID)
- [ ] Translations ≥ 95% for `id`

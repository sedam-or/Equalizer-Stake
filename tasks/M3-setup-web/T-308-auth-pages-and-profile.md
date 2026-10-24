---
id: T-308
title: Add the auth pages (login, TOTP step, forgot, reset, invite accept) and the /me profile page
milestone: M3
status: todo
depends_on: [T-307, T-303]
spec_refs: [IDN-02, IDN-06, IDN-07, IDN-08, IDN-12, SCH-16, FE-36]
estimated_size: M
---

## Goal
People get in, recover and manage their account without help. The pages are friendly on a phone, and they never reveal whether an email exists.

## Context
- `/login`:
  - Tenant logo and name from `/meta/app`. Email + password (show/hide toggle). "Forgot password?" link.
  - Submit → `POST /auth/login`. Error handling:
    - 401 `invalid_credentials` → "Email or password is incorrect."
    - 423 `locked` → "Too many attempts. Try again at {time}."
    - 401 `totp_required` → show the 6-digit code step (one input with `autocomplete="one-time-code"`, plus a "Use a recovery code" link).
    - `needs_totp_setup` → after login, redirect to `/me#security` with a banner.
  - Multiple roles → after login, go to `/dashboard` with the default role (server picked); the role switcher is in the shell.
  - `?next=` is honoured, same-origin paths only.
- `/forgot`: email → always shows "If an account exists, we sent a link." (202).
- `/reset/:token`: new password + confirm. Live hint for the minimum length. **Backend addition in this task:** `/meta/app` gains `auth: {password_min_length, totp_policy}` (public, from IdentityPolicy). 400 `token_invalid` → "This link has expired. Request a new one." with a link to `/forgot`.
- `/invite/:token`: `GET /auth/invites/{token}` → "Welcome to {tenant}, {display_name}. You're joining as {role term}." Set password → accept → logged in → `/dashboard`. 410 → "This invite has expired. Ask your school to send a new one."
- The language switcher (en/id) on public pages is stored in localStorage `eq:lang` (a UI preference).
- `/me` (tabs):
  - **Profile:** display name, phone, locale, and avatar upload (`POST /files` purpose avatar, then `PATCH /me` with If-Match).
  - **Security:** change password; TOTP setup (QR svg from the server, confirm code, then show 10 recovery codes once with "Download .txt" and "I saved them"); disable TOTP (password prompt); sessions list with "Sign out" per device and "Sign out everywhere else".
  - **Preferences:** theme (system/light/dark), density (auto/compact/comfortable).
  - **Calendar feed:** create link, copy, revoke (SCH-16 endpoints from T-211).
- Copy is in `modules.identity` (en, id). Use the voice from spec 09 §6: plain, no blame.
- Forms: react-hook-form + zod. Errors are linked with `aria-describedby`. Submit buttons show a loading state. On mobile, the primary button is in a sticky bottom bar.

## Scope
### In
- The routes and pages above, the `/meta/app.auth` addition (backend + OpenAPI regen), and the locale strings.

### Out
- Notification preferences (M5). Privacy export (M6).

## Acceptance criteria
- AC1. Playwright: login with the dev seed admin → dashboard. Wrong password → the generic message. → `web/e2e/auth.spec.ts › login`
- AC2. TOTP flow end to end: enable, log out, log in with a code (the test computes it with `otplib` from the secret shown), and a recovery code works once. → `totp`
- AC3. Invite: the admin creates a person with an invite link (API), the e2e opens the link, sets a password, and lands on the dashboard. → `invite accept`
- AC4. `/forgot` with an unknown email shows the same message as a known one. → `forgot no enumeration`
- AC5. axe: 0 serious or critical on every auth page and every `/me` tab.

## Tests to write
| File | Test names |
|---|---|
| `web/e2e/auth.spec.ts` | AC1–AC4 |
| `web/src/modules/identity/**/*.test.tsx` | form validation units, a11y |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Web checks + E2E pass; `make api` regenerated
- [ ] Commit `feat(web): auth pages and profile [T-308]`, signed off

## Report

## Questions

# ADR-0016: In-process auth with a short in-memory JWT and a rotating refresh cookie

- Status: Proposed (2026-10-25), accepted on merge of the PR that adds `spec/25-auth-and-access-control.md`

## Context
Ritma is one binary plus PostgreSQL (ADR-0001/0002, OSS-05). It runs in the cloud, on a 1-click host, on a school PC behind a tunnel, and in Docker. Users are staff on desktops and parents on phones, often on shared or old devices. The SPA is served from the same origin as the API (ADR-0003).

Options considered:
1. **Server-side session cookie only** (opaque id, session row looked up on every request). Simple and revocable, but every API call needs CSRF protection and a DB or cache lookup, and the desktop/tunnel setups make cookie scoping fragile.
2. **Long-lived JWT in `localStorage`.** Stateless, but any XSS steals a token valid for days, and revocation is impossible.
3. **External identity provider** (Keycloak, Ory, Zitadel). Strong features, but a second stateful service to install, back up and upgrade on a school PC.
4. **Short JWT in memory plus an opaque, rotating refresh token in an `HttpOnly; SameSite=Strict` cookie scoped to `/api/v1/auth`.**

## Decision
Option 4 (IDN-03/04/05, spec 25 §3):
- Access token: HS256 JWT, 10 minutes, memory only, with `ver` so a password change kills every token at once.
- Refresh token: 32 random bytes, only its hash stored, rotated on every use, reuse of a rotated token revokes the whole family.
- Only the three cookie endpoints need CSRF (`X-EQ-CSRF` + `Origin`); every other endpoint is bearer-only.
- Built from `argon2`, `jsonwebtoken`, `totp-rs` and `eq_core::crypto`; no auth server.

## Consequences
- ✅ XSS can't read the refresh token, and a stolen access token lives at most 10 minutes.
- ✅ Revocation works: logout, password change, deactivation and reuse detection end sessions within one token lifetime.
- ✅ No extra service on any distribution channel.
- ❌ A page reload needs one refresh round trip before the first API call (FE-10). Accepted.
- ❌ We own the auth code. Mitigated by audited libraries, the identity ACs (lockout, reuse, enumeration timing), ZAP and fuzzing (T-711), and CODEOWNERS on `crates/eq-identity/`.
- ❌ SSO (OIDC/SAML) for schools isn't covered. Cloud signup adds Google OIDC (T-903) on top of the same session model.

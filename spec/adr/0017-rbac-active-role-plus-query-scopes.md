# ADR-0017: RBAC with one active role, static default grants and query scopes

- Status: Proposed (2026-10-25), accepted on merge of the PR that adds `spec/25-auth-and-access-control.md`

## Context
Small schools have five recognisable roles (owner, admin, instructor, guardian, student), and one person often holds several (a teacher who is also a parent). Most access questions are about *rows* ("only my children", "only my sessions"), not about actions. The v0 capstone mixed role checks into handlers and had two competing role models.

Options considered:
1. **Union of all the person's roles.** Convenient, but a teacher-parent would see other families' data through the instructor grants while acting as a parent, and screens get confusing.
2. **Fully configurable roles and permissions per tenant** (custom roles, a permission editor). Flexible, but a large support and testing surface, and it breaks the "3 steps" promise.
3. **ABAC / policy engine** (Cedar, OPA, Casbin). Powerful, but it adds a language and runtime for rules that are simple and fixed.
4. **Static role → permission grants declared by each module, one active role at a time, plus row scopes applied in SQL.**

## Decision
Option 4 (IDN-09/10/11, spec 25 §5–§7):
- Permissions are strings `<module>.<entity>.<action>`, declared with default grants by the module that owns them (`PermissionDef`). The effective set is the grants of the **active role**, across enabled modules.
- A person switches roles explicitly (`/auth/switch-role`), and the UI changes with the role.
- Row access is a `Scope` (`All`, `Instructor`, `Guardian`, `Self_`) applied in the repository query. Out-of-scope rows return 404; a missing permission returns 403.
- Policy-dependent options (`payer_can_cancel`, `marked_by`) are domain guards, not permissions.
- Tenant-editable grants are limited to the documented grantable permissions (for example `reports.revenue.read`) in v1.

## Consequences
- ✅ The complete matrix fits on one page (spec 25 §6) and is tested against the code (`rbac_matrix_matches_spec`, TST-04).
- ✅ Scope bugs can't leak rows through a forgotten Rust filter, because scope is part of the SQL.
- ❌ Schools can't invent roles in v1 (for example "receptionist with no billing"). Revisit with an RFC once real demand exists.
- ❌ Multi-role people must switch roles. Accepted; it matches how they think ("now I'm acting as a parent").

# ADR-0007: Money as i64 minor units + ISO 4217 currency; percentages as basis points

- Status: Accepted (2026-10-25)

## Decision
- Money is `Money { amount_minor: i64, currency }`.
- No floats anywhere in billing; clippy enforces it.
- Percentages are `Bps(u32)`.
- Rounding is half-up only at line level, in one function.
- The tenant currency is fixed once any invoice exists.

## Consequences
- ✅ Exact sums, a DB CHECK on totals, and property-tested arithmetic.
- ❌ Multi-currency tenants aren't supported in v1 (not needed by the ICP).

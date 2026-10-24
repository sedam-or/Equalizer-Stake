# 04 — Policy Engine: business rules as typed, versioned data

## 1. Principle (first principles, ADR-0004)

Every "business rule" found in v0 turned out to be a **parameter of a known mechanism**: a day of the month, an amount, a radius, a list of opening hours, a tier table. None of them needed arbitrary logic. So:

**POL-01.** Business rules are **typed Rust structs**, one per module (`XPolicy`). They are serialised as JSON, stored append-only and versioned per tenant. There is no rule DSL, no scripting and no expression language.

**POL-02.** **No business value is ever a literal in code.** If code needs a number, day, duration, amount, percentage, radius or time-of-day that a school could reasonably want to differ, it MUST come from a policy field. Every task's Definition of Done includes a grep check for this. Allowed literals: technical constants (page sizes, crypto parameters, protocol limits) declared in `const` items with a `// technical:` comment.

## 2. Storage

```sql
CREATE TABLE policies (
  tenant_id       uuid        NOT NULL REFERENCES tenants(id),
  module_id       text        NOT NULL,
  version         int         NOT NULL CHECK (version >= 1),
  schema_version  int         NOT NULL,
  effective_from  timestamptz NOT NULL,
  document        jsonb       NOT NULL,
  note            text,
  created_by      uuid        REFERENCES persons(id),
  created_at      timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, module_id, version)
);
CREATE INDEX policies_lookup ON policies (tenant_id, module_id, effective_from DESC);
-- append-only: no UPDATE/DELETE grants for eq_app (SEC-07)
```

**POL-03.** Writes are append-only. A change inserts `version = max + 1`. Rows are never updated or deleted, so the full history is the audit trail of rule changes. Each insert is also recorded in the audit log with the before and after documents.

## 3. Reading policies

```rust
// eq-core::policy
pub trait PolicyDoc: Serialize + DeserializeOwned + JsonSchema + Default + Clone + Send + Sync + 'static {
    const MODULE: ModuleId;
    const SCHEMA_VERSION: u32;
    fn validate(&self) -> Result<(), Vec<FieldError>>;   // cross-field rules (POL-10)
    fn summarize(&self) -> Vec<SummaryLine>;              // POL-07
}

pub struct PolicyStore { /* pool + cache */ }
impl PolicyStore {
    /// Policy in force at instant `at` (latest version with effective_from <= at).
    pub async fn get<P: PolicyDoc>(&self, tx: &mut Tx, tenant: TenantId, at: Timestamp) -> Result<Versioned<P>>;
    /// Convenience: at = clock.now()
    pub async fn current<P: PolicyDoc>(&self, tx: &mut Tx, tenant: TenantId) -> Result<Versioned<P>>;
    /// Append a new version; validates schema + P::validate(); writes audit + emits core.policy_changed.v1.
    pub async fn put<P: PolicyDoc>(&self, tx: &mut Tx, tenant: TenantId, doc: P, effective_from: Timestamp, actor: &Actor, note: Option<String>) -> Result<Versioned<P>>;
}
pub struct Versioned<P> { pub version: i32, pub effective_from: Timestamp, pub doc: P }
```

**POL-04.** Cache: `(tenant, module) → Vec<Versioned<Value>>`, invalidated by `NOTIFY eq_policy_changed`. A read with no row falls back to `P::default()` merged with the tenant preset. That situation means an integrity problem, so it also logs `warn!`.

**POL-05. Effective dating.**
- `effective_from` defaults to `now()`. The owner may choose a future date in the UI: "Apply from: [Now] [Start of next month] [Pick date]".
- **Terms snapshot rule:** when a policy produces a long-lived commitment, the relevant terms are **copied onto that record** at creation. Examples: an invoice stores `due_at`, `late_fee_terms` and `overdue_terms`; a reservation stores `cancel_terms`; an enrollment stores `leave_terms`. Later policy changes never retroactively change existing commitments. This is the main protection against "the owner changed the fee and old invoices changed".
- Calculations that aren't commitments (opening hours, buffer, conflict checks) use the policy in force at the **time of the thing being checked** (e.g. the session start), via `get(at = session.starts_at)`.

## 4. Schema, UI hints and forms

**POL-06.** Each policy struct derives `schemars::JsonSchema`. UI hints are attached with `#[schemars(extend("x-ui" = ...))]`. The SPA renders the whole form from the schema (FE-20), so **adding a rule field needs no frontend code**, unless it needs a new widget.

`x-ui` object keys:

| Key | Type | Meaning |
|---|---|---|
| `group` | `"essentials" \| "advanced"` | Which collapsible section in setup step 3 |
| `order` | int | Sort order within the group |
| `widget` | enum, below | Renderer override |
| `unit` | `"minutes" \| "hours" \| "days" \| "meters" \| "percent" \| "money"` | Suffix and formatting |
| `visible_if` | `{ "path": "/gps/enabled", "equals": true }` | Conditional display (JSON Pointer relative to the module doc) |
| `requires_module` | ModuleId | Hide the field if that module is disabled |

Widgets: `toggle`, `number`, `money` (minor units ↔ formatted major, tenant currency), `percent` (basis points ↔ %), `duration` (minutes), `time` ("HH:MM"), `weekday-hours` (7-day editor with multiple intervals), `day-of-month` (1–28), `tiers` (editable table), `chips` (number list), `select`, `textarea-i18n` (per-locale text), `map-pin` (lat/lng with map and "use my location"), `offering-ref` (select from catalog by key), `hidden` (not rendered by `PolicyForm`; edited by a dedicated screen, e.g. branding).

Labels and help come from i18n keys derived from the JSON Pointer: `policy.<module>.<path with dots>.label` / `.help`. Example: `policy.billing.late_fee.amount_minor.label`. **CI fails if any schema path lacks `en` and `id` labels** (TST-09).

**POL-07. Summaries.** `summarize()` returns `Vec<SummaryLine { key: &'static str, args: serde_json::Map<String, Value> }>`. The SPA renders each line with ICU MessageFormat (`policy.billing.summary.cycle`: "Invoices are issued on the {issue_day, selectordinal, ...} and due on the {due_day, selectordinal, ...}."). Money args are `{ "$money": { "amount_minor": 10000000, "currency": "IDR" } }`. Duration args are `{ "$minutes": 45 }`. The SPA formats them with the tenant locale.

**POL-08. Schema evolution.**
- Additive changes (a new field with `#[serde(default)]`) don't bump `SCHEMA_VERSION`.
- Renames, type changes and removals MUST bump it and implement `Module::migrate_policy(from, doc)`.
- At startup, the server migrates every tenant's latest stored document lazily on first read (in memory). On the next `put`, it is stored at the new version.
- `RELEASING.md` lists schema bumps in the changelog.

**POL-13. Enum encoding.** Policy enums are serialised externally tagged in `snake_case`: unit variants as strings (`"skip"`), data variants as single-key objects (`{"flat": {"amount": 10000000}}`). The preset YAML uses the same encoding.

**POL-09. Preset merge.** A preset contains, per module, a **JSON Merge Patch (RFC 7386)** applied on top of `P::default()`. Arrays are replaced, not merged. The result MUST pass `validate()`. CI tests every preset × module (TST-08).

**POL-10. Validation layers.** Order:
1. JSON Schema (`jsonschema` crate, draft 2020-12). This checks types, ranges and enums.
2. `serde` deserialization into `P`.
3. `P::validate()`, which handles cross-field rules.

Errors are returned as `FieldError { pointer: "/cycle/due_day", code: "must_be_gte", params: {"other": "/cycle/issue_day"} }`. The client runs the same JSON Schema with `ajv` for instant feedback, and the server is authoritative.

## 5. Common value types (in `eq-core::policy::types`)

```rust
pub struct LocalTime(chrono::NaiveTime);            // serde "HH:MM", minute precision
pub struct TimeInterval { pub start: LocalTime, pub end: LocalTime }   // start < end, same day
pub enum Weekday { Mon, Tue, Wed, Thu, Fri, Sat, Sun }
pub struct DayHours { pub weekday: Weekday, pub open: bool, pub intervals: Vec<TimeInterval> } // non-overlapping, sorted
pub struct WeeklyHours(pub [DayHours; 7]);           // validate: weekday order Mon..Sun
pub struct Bps(pub u32);                             // 0..=10000 unless documented otherwise
pub struct MinorAmount(pub i64);                     // >= 0; currency = tenant currency
pub struct DayOfMonth(pub u8);                       // 1..=28 (no 29–31: every month has it)
pub struct I18nText(pub BTreeMap<Locale, String>);   // at least the tenant default locale
pub struct GeoPoint { pub lat: f64, pub lng: f64 }   // only non-money floats allowed
```

**POL-11.** `DayOfMonth` is capped at 28 so that "every month" is always defined. That covers v0's 5/10/11 values.

**POL-12.** Money in policies is `MinorAmount` in the **tenant currency** (`CorePolicy.currency`). Changing currency after any invoice exists is blocked (`PolicyViolation rule=core.currency.locked`).

## 6. Policy catalog (normative, v1)

Each block shows the Rust shape, the **generic default** (used by the "Blank" preset and as the merge base), bounds, and the `x-ui` group (E = essentials, A = advanced). Preset values are in `05-presets-and-setup.md`.

### 6.1 `core` — CorePolicy (step 1 sets most of this)

```rust
pub struct CorePolicy {
    pub locale: Locale,                      // E  "en" | "id"                         default "en"
    pub currency: Currency,                  // E  ISO 4217 (supported list in core)  default "USD"
    pub timezone: Tz,                        // E  IANA                                default "UTC"
    pub week_starts_on: Weekday,             // A  Mon | Sun                           default Mon
    pub terminology: Terminology,            // A  label overrides                     default {}
    pub privacy: PrivacyPolicy,              // A
    pub branding: Branding,                  // A (edited in Settings → Branding, not shown in setup step 3)
}
pub struct Branding {
    pub brand_hue: u16,                      // 0..=359, default 255 (DS-10)
    pub corner_style: CornerStyle,           // Sharp | Default | Round, default Default
    pub logo_file_id: Option<Uuid>,          // files.purpose = logo; None = product logo
}
pub struct Terminology { pub overrides: BTreeMap<TermKey, I18nText> }
pub enum TermKey { Instructor, Guardian, Student, Offering, Session, Resource, Location, Enrollment }
pub struct PrivacyPolicy {
    pub deletion_sla_business_days: u16,     // 1..=30, default 14 (UU PDP Art. 19 practice)
    pub block_deletion_with_open_invoices: bool, // default true
    pub photo_retention_days: u16,           // attendance photos; 7..=3650, default 365
}
```

Step 1's country selection fills in `locale`, `currency` and `timezone`, using `core/countries.json` (ISO 3166 → default locale, currency and primary timezone).

### 6.2 `identity` — IdentityPolicy

```rust
pub struct IdentityPolicy {
    pub idle_timeout_minutes: RoleMap<u16>,  // A 5..=1440; default owner 30, admin 30, instructor 60, guardian 120, student 120
    pub absolute_session_days: u16,          // A 1..=90, default 30
    pub lockout: Lockout,                    // A
    pub password_min_length: u8,             // A 8..=64, default 10
    pub invite_expires_days: u16,            // A 1..=30, default 7
    pub reset_token_ttl_minutes: u16,        // A 10..=1440, default 60
    pub reset_max_per_hour: u8,              // A 1..=10, default 3
    pub totp: TotpPolicy,                    // A
}
pub struct Lockout { pub max_failed: u8 /*3..=20, 5*/, pub window_minutes: u16 /*1..=120, 15*/, pub lock_minutes: u16 /*1..=1440, 15*/, pub notify_admins: bool /*true*/ }
pub enum TotpPolicy { Off, OptionalForStaff, RequiredForOwnerAdmin }   // default OptionalForStaff
pub struct RoleMap<T> { pub owner: T, pub admin: T, pub instructor: T, pub guardian: T, pub student: T }
```

### 6.3 `scheduling` — SchedulingPolicy

```rust
pub struct SchedulingPolicy {
    pub opening_hours: WeeklyHours,          // E widget weekday-hours. default Mon–Fri 09:00–21:00, Sat 09:00–17:00, Sun closed
    pub buffer_minutes: u16,                 // E 0..=120, default 10   (applies after each session, per resource AND instructor)
    pub slot_step_minutes: u16,              // A one of 5,10,15,30,60, default 15
    pub default_durations_minutes: Vec<u16>, // E chips, each 10..=480, 1..=8 items, default [30,45,60]
    pub booking_window: BookingWindow,       // A
    pub holidays: HolidayRules,              // A
    pub recurrence: RecurrenceRules,         // A (visible only if enrollment enabled)
    pub cancellation: CancellationRules,     // E
    pub alternatives: AlternativeSearch,     // A
}
pub struct BookingWindow { pub min_notice_hours: u16 /*0..=720, 0*/, pub max_days_ahead: u16 /*1..=365, 90*/ }
pub struct HolidayRules {
    pub on_closure: ClosureAction,           // default ShiftToNextWeek
    pub when_no_slot: NoSlotAction,          // default MakeupCredit
}
pub enum ClosureAction { Skip, ShiftToNextWeek, MakeupCredit }
pub enum NoSlotAction { Skip, MakeupCredit, Substitute }
pub struct RecurrenceRules {
    pub materialize_weeks_ahead: u8,         // 1..=26, default 6
    pub sessions_per_cycle: Option<u8>,      // None = every weekly occurrence is regular; Some(4) = cap per billing month
    pub extra_occurrence: ExtraOccurrence,   // what to do with occurrence #(n+1) in a month (e.g. the 5th week)
}
pub enum ExtraOccurrence {
    Regular,                                 // default
    Skip,
    Substitute { offering_key: String, duration_minutes: u16, consumes_credit: bool },
}
pub struct CancellationRules {
    pub payer_can_cancel: bool,              // default true
    pub notice_tiers: Vec<NoticeTier>,       // E widget tiers. default [{min_hours_before: 24, makeup_credit_bps: 10000}]
                                             // sorted desc by min_hours_before; below the last tier = 0 credit
    pub staff_cancel_credit_bps: Bps,        // default 10000 (school cancels → full make-up credit)
    pub instructor_absence: InstructorAbsence, // default MakeupCredit
    pub student_no_show: NoShow,             // E default Forfeit
    pub reason_required: bool,               // default true
}
pub struct NoticeTier { pub min_hours_before: u16, pub makeup_credit_bps: Bps }
pub enum InstructorAbsence { SubstituteThenReschedule, Reschedule, MakeupCredit }
pub enum NoShow { Forfeit, MakeupCredit }
pub struct AlternativeSearch { pub count: u8 /*1..=10, 3*/, pub radius_minutes: u16 /*15..=480, 60*/ }
```

Validation:
- Opening intervals are within 00:00–24:00, ordered and non-overlapping.
- At least one open day.
- Every `default_durations_minutes` value is a multiple of `slot_step_minutes`.
- Notice tiers have strictly descending `min_hours_before`.
- `Substitute.offering_key` must exist in the catalog when the policy is saved (checked in the service, not the struct).

### 6.4 `enrollment` — EnrollmentPolicy

```rust
pub struct EnrollmentPolicy {
    pub registration: RegistrationRules,     // E
    pub guardians: GuardianRules,            // E
    pub age: AgeRules,                       // A
    pub leave: LeaveRules,                   // E
    pub makeup_credits: MakeupCreditRules,   // A
    pub eligibility: EligibilityRules,       // A (e.g. exam eligibility; v0 FR-07)
}
pub struct RegistrationRules {
    pub public_form: bool,                   // default true  (/join page)
    pub approval: Approval,                  // default AdminApproval  { Auto, AdminApproval }
    pub placement: Placement,                // default None  { None, Trial, Audition }
    pub collect_address: FieldReq,           // default Optional { Off, Optional, Required }
    pub collect_birth_date: FieldReq,        // default Required
    pub payment_proof: FieldReq,             // default Off
    pub consent_text: I18nText,              // E privacy/terms consent shown on form, default generic text
}
pub struct GuardianRules {
    pub required_under_age_years: u8,        // 0..=25, default 18 (0 = never required)
    pub invoices_to_primary_only: bool,      // default true
}
pub struct AgeRules { pub enforcement: AgeEnforcement /* Block | Warn | Off, default Block */ }
pub struct LeaveRules {
    pub enabled: bool,                       // default true
    pub max_days_per_year: u16,              // 1..=365, default 30
    pub min_notice_days: u16,                // 0..=60, default 0
    pub fee: MinorAmount,                    // default 0  (v0 "bea administrasi")
    pub billing_during_leave: LeaveBilling,  // default Suspend { Suspend, ChargeFull, ChargeBps(Bps) }
    pub over_limit: OverLimit,               // default Block { Block, EndAndRequireReregistration }
}
pub struct MakeupCreditRules { pub enabled: bool /*true*/, pub expires_after_days: u16 /*7..=365, 60*/, pub max_outstanding: u8 /*1..=20, 4*/ }
pub struct EligibilityRules {
    pub enabled: bool,                       // default false
    pub label: I18nText,                     // default "Exam eligibility"
    pub min_tenure_months: u8,               // 0..=36, default 3
    pub min_attendance_bps: Bps,             // default 9000
    pub max_no_shows: u8,                    // default 1
    pub window_months: u8,                   // 1..=12, default 3
    pub min_sessions_for_data: u8,           // default 5 (below → "insufficient data")
}
```

### 6.5 `billing` — BillingPolicy

```rust
pub struct BillingPolicy {
    pub cycle: Cycle,                        // E
    pub grouping: Grouping,                  // A default PerPayer { PerPayer, PerEnrollment }
    pub first_cycle: FirstCycle,             // A default ProratedBySessions { Full, ProratedBySessions, StartNextCycle }
    pub payment_reference: PaymentReference, // A
    pub partial_payment: PartialPayment,     // A
    pub late_fee: LateFee,                   // E
    pub overdue: OverdueRules,               // E
    pub reminders_days_from_due: Vec<i16>,   // A chips, -30..=30, default [-3, 0, 1]
    pub payment_methods: Vec<PaymentMethodCfg>, // E
    pub tax: TaxRules,                       // A
    pub numbering: Numbering,                // A
}
pub struct Cycle { pub issue_day: DayOfMonth /*default 1*/, pub due_day: DayOfMonth /*default 10*/ }   // validate due_day >= issue_day
pub enum PaymentReference { None, UniqueCode { digits: u8 /*2..=4, 3*/ }, UniqueAmount { max_minor: i64 /*1..=99_900, 99_900*/ } } // default None
pub struct PartialPayment { pub allowed: bool /*true*/, pub min_first_payment_bps: Bps /*default 0*/ }
pub struct LateFee {
    pub enabled: bool,                       // default false
    pub kind: LateFeeKind,                   // Flat { amount: MinorAmount } | Percent { bps: Bps }   default Flat{0}
    pub days_after_due: u8,                  // 1..=60, default 1  (the day after the due day)
    pub once_per_invoice: bool,              // default true (v1 only supports true; false reserved)
}
pub struct OverdueRules {
    pub action: OverdueAction,               // default None { None, Restrict, Freeze }
    pub days_after_due: u8,                  // 1..=60, default 1
    pub unfreeze: Unfreeze,                  // default AutoOnFullPayment { AutoOnFullPayment, AdminManual }
    pub skip_issuing_while_frozen: bool,     // default true
}
pub struct PaymentMethodCfg { pub kind: PaymentMethodKind /* BankTransfer | Cash | Qris | EWallet | Other */, pub enabled: bool, pub instructions: I18nText }
pub struct TaxRules { pub enabled: bool /*false*/, pub name: I18nText /*"Tax"*/, pub rate_bps: Bps /*0*/, pub inclusive: bool /*true*/ }
pub struct Numbering { pub prefix: String /*"INV", 1..=8 [A-Z0-9]*/, pub reset: NumberReset /* Monthly | Yearly | Never, default Yearly */ }
```

Semantics (normative):
- **Restrict:** the payer's students keep attending existing sessions, but the payer can't create new rental reservations or new enrollments, and the portal shows a banner.
- **Freeze:**
  - It applies at the **household** level, meaning the payer and every student they pay for (v0 SRS-09).
  - Future sessions for those students keep their slots but participants are marked `blocked`, and the instructor sees a "frozen" badge.
  - Rental bookings are disabled and the portal is read-only except Invoices/Pay.
- **Unfreeze AutoOnFullPayment:** happens when *all* overdue invoices of the payer reach `paid`.

### 6.6 `attendance` — AttendancePolicy

```rust
pub struct AttendancePolicy {
    pub instructor_checkin: CheckinRules,    // E
    pub review: ReviewRules,                 // A
    pub student_marks: StudentMarkRules,     // E
}
pub struct CheckinRules {
    pub enabled: bool,                       // default false
    pub window_before_start_minutes: u16,    // 0..=240, default 30
    pub window_after_start_minutes: u16,     // 0..=240, default 15
    pub gps: GpsCheck,                       // { enabled: bool /*true*/, radius_m: u16 /*25..=5000, 150*/, jitter_allowance_m: u16 /*0..=100, 15*/ }
    pub photo: PhotoCheck,                   // { required: bool /*true*/, max_bytes: u32 /*2_000_000*/, min_width: u16 /*640*/, min_height: u16 /*480*/ }
    pub face: FaceCheck,                     // { enabled: bool /*false*/, min_face_ratio_bps: Bps /*3000*/ }  (client-side; ATT-06)
    pub on_failed_check: FailedCheck,        // default PendingReview { PendingReview, Reject }
}
pub struct ReviewRules { pub sla_minutes: u16 /*15..=2880, 120*/, pub remind_after_minutes: u16 /*60*/, pub escalate_to_owner: bool /*true*/ }
pub struct StudentMarkRules {
    pub enabled: bool,                       // default true
    pub marked_by: MarkedBy,                 // default Instructor { Instructor, Admin, Either }
    pub lock_after_hours: u16,               // 1..=720, default 48
    pub late_after_minutes: u16,             // 0..=60, default 10 (status "late" option)
}
```

The GPS reference point is `locations.geo` (data, not policy). If GPS is enabled and the session's location has no `geo`, the check is skipped and recorded as `gps: not_configured`. The rules screen shows a warning with a link to set the pin.

### 6.7 `rental` — RentalPolicy

```rust
pub struct RentalPolicy {
    pub booking_types: Vec<BookingType>,     // E widget tiers-like list, 1..=10
    pub hours: RentalHours,                  // A default SameAsScheduling { SameAsScheduling, Custom(WeeklyHours) }
    pub payment: RentalPayment,              // E
    pub who_can_book: WhoCanBook,            // A default Members { Members, MembersAndGuests }
}
pub struct BookingType {
    pub key: String,                         // slug, unique
    pub name: I18nText,
    pub approval: Approval,                  // Auto | AdminApproval
    pub min_minutes: u16, pub max_minutes: u16, pub step_minutes: u16,
    pub min_notice_hours: u16, pub max_days_ahead: u16,
    pub cancel_until_hours_before: u16, pub cancel_refund_bps: Bps,
}
// default: [{ key:"standard", name:"Standard booking", approval:Auto, min:60, max:240, step:60, min_notice:2, max_days:14, cancel_until:24, refund:10000 }]
pub struct RentalPayment {
    pub mode: RentalPaymentMode,             // default PayLater { PayLater, DepositToConfirm{bps}, FullToConfirm }
    pub hold_minutes: u16,                   // 5..=2880, default 60: unpaid holds expire
}
```

Hourly prices are **catalog data**: `rate_cards` (effective-dated, per resource × booking type, optional peak windows). They are not policy, because owners think of prices as a price list, not as rules.

### 6.8 `notify` — NotifyPolicy

```rust
pub struct NotifyPolicy {
    pub routing: Routing,                    // E { critical: Vec<Channel> /*[Email, WhatsApp]*/, standard: Vec<Channel> /*[WhatsApp, Email]*/ }
    pub retry: Retry,                        // A { attempts_per_channel: u8 /*1..=5, 3*/, backoff_seconds: Vec<u16> /*[10,20,40]*/ }
    pub circuit_breaker: Breaker,            // A { failure_threshold: u8 /*5*/, cooldown_minutes: u16 /*30*/ }
    pub session_reminders_minutes_before: Vec<u16>, // E chips, default [60], empty = off
    pub quiet_hours: QuietHours,             // A { enabled: false, start "21:00", end "07:00" } — standard msgs deferred
    pub aggregate_per_payer: bool,           // A default true (one message per guardian listing all students)
}
pub enum Channel { Email, WhatsApp, InApp }  // InApp is always delivered additionally
```

Channel **credentials** (SMTP password, WhatsApp token) are **not** policy. They live in `channel_configs` (encrypted, SEC-03) and are managed under Settings → Channels.

Categories: `critical` covers invoice issued, overdue, freeze, cancellation of a session within 24 h, and security. `standard` covers everything else.

### 6.9 `audit` — AuditPolicy

```rust
pub struct AuditPolicy {
    pub retention_years: u8,                 // A 1..=10, default 7
    pub anonymize_pii_after_years: u8,       // A 1..=10, default 2; must be <= retention_years
    pub admin_categories: Vec<AuditCategory>,// A default [Operational]  { Operational, Security, Compliance }
}
```

### 6.10 `reports` — ReportsPolicy

```rust
pub struct ReportsPolicy {
    pub widgets: Vec<WidgetKey>,             // A default all available for enabled modules
    pub default_range: DefaultRange,         // A default CurrentMonth { CurrentMonth, Last30Days, CurrentYear }
    pub min_sessions_for_rate: u8,           // A default 5 (fewer → value marked "*")
}
```

### 6.11 `feedback` — FeedbackPolicy

```rust
pub struct FeedbackPolicy {
    pub progress_notes: ProgressNotes,       // E { enabled: true, categories: Vec<NoteCategory>, publish: Immediate | AfterAdminReview (default Immediate) }
    pub ratings: Ratings,                    // E { enabled: true, min_sessions_before: u8 /*1*/, anonymous_to_instructor: bool /*true*/ }
}
pub struct NoteCategory { pub key: String, pub label: I18nText, pub scale_max: u8 /*0 = no rating, else 3..=10*/ }
// default categories: progress(5), behaviour(5), attendance(5), recommendation(0)
```

## 7. Settings API (shared by setup step 3 and Settings → Rules)

| Method | Path | Perm | Body / Result |
|---|---|---|---|
| GET | `/api/v1/settings/policies` | `core.policy.read` | `[{module, version, effective_from, schema, document, summary, pending_future?}]` for enabled modules |
| GET | `/api/v1/settings/policies/{module}` | `core.policy.read` | One of the above, plus `history: [{version, effective_from, created_by, note}]` |
| GET | `/api/v1/settings/policies/{module}/versions/{v}` | `core.policy.read` | The full document |
| POST | `/api/v1/settings/policies/{module}/preview` | `core.policy.write` | `{document}` → `{valid, errors, summary}`. Validates and summarises without saving (used live by the form, debounced 300 ms) |
| PUT | `/api/v1/settings/policies/{module}` | `core.policy.write` | `{document, effective_from?, note?}` → the new `Versioned`. Requires `If-Match: <current version>` (409 on mismatch) |

Only `owner` has `core.policy.write` by default.

## 8. What "≤ 3 steps" means for rule changes later

Change a rule = (1) Settings → Rules → module card, (2) edit the field and read the live summary, (3) Save, choosing "now" or a date. Each saved change is one audit entry and one version.

## 9. Legacy → policy translation (complete) and contradiction log

| # | v0 rule (source) | v2 location | Music-school preset value | Notes |
|---|---|---|---|---|
| 1 | Hours Mon–Fri 14:30–17:00, Sat 08:00–12:00, Sun closed (addendum §10) | `scheduling.opening_hours` | exactly that | **Contradiction:** v0 code used 08–21 and 09–21, PRODUCT.md used Mon–Sat 08–20. Resolved in favour of the owner-confirmed addendum |
| 2 | 10-minute break between sessions (FR-02) | `scheduling.buffer_minutes` | 10 | Now applies to resource **and** instructor holds |
| 3 | Durations 30/45 min; 45 standard, piano beginner 30 (addendum §11) | `scheduling.default_durations_minutes` + per-offering `durations` | [30, 45] | Per-offering duration lives in the catalog |
| 4 | No scheduling on public holidays; reschedule to next week or convert to theory (FR-12) | `scheduling.holidays` + `closures` data | on_closure **Skip** (when_no_slot irrelevant) | **Contradiction:** the addendum says "holidays: no replacement". The preset follows the addendum: `on_closure: Skip`. FR-12 behaviour is still available as an option |
| 5 | 5th week of the month = one theory class, 30 min, non-credit (FR-12, addendum §2) | `scheduling.recurrence.sessions_per_cycle=4`, `extra_occurrence=Substitute{offering_key:"theory_class", 30, consumes_credit:false}` | as stated | |
| 6 | Cancellation credit admin 100% / guardian ≥24 h 50% / <24 h 0% (FR-02 §2.5) | `scheduling.cancellation` | staff 10000; tiers `[]`; no_show Forfeit | **Contradiction:** the addendum §2 says student absence forfeits and there are no refund tiers. The preset follows the addendum (empty tiers = cancelling payer gets 0). FR-02 tiers remain expressible |
| 7 | Instructor absent → substitute instructor or another day (addendum §2) | `scheduling.cancellation.instructor_absence` | SubstituteThenReschedule | |
| 8 | Concurrent saves → "slot already taken" (FR-02 §2.4) | Not a policy: DB exclusion constraint (SCH-03) | — | |
| 9 | Show alternative slots on conflict (FR-02, SRS-08 §4.2: 3 alternatives, ±1 h) | `scheduling.alternatives` | count 3, radius 60 | |
| 10 | Draft autosave every 30 s, server TTL 7 days (FR-08) | Not a policy: technical constants in the SPA (FE-30) and `drafts` TTL in core | — | Not a business rule |
| 11 | Invoice on the 5th, due 10th 23:59, fine and freeze on the 11th (FR-03/04, addendum §1) | `billing.cycle {issue_day:5, due_day:10}`, `late_fee {Flat 10_000_000, days_after_due:1}`, `overdue {Freeze, days_after_due:1, AdminManual}` | as stated | Due means end of `due_day` in tenant TZ (23:59:59.999) |
| 12 | 1 invoice per guardian, N students combined (FR-03, SRS-09) | `billing.grouping = PerPayer` | PerPayer | |
| 13 | 3-digit unique code, max 3 collision retries, then manual (FR-03 §2.3) | `billing.payment_reference = UniqueCode{3}` | UniqueCode{3} | Uniqueness scope: open invoices of the tenant. After 3 retries the invoice is `needs_manual_reference` |
| 14 | DP ≥70% allowed until the 10th (FR-03) vs "partial payment NOT allowed" (SRS-09 §5) | `billing.partial_payment {allowed, min_first_payment_bps}` | allowed, 7000 | **Contradiction.** Resolved in favour of FR-03 (approved, more specific). A partial payment does **not** prevent the late fee unless the invoice is fully paid by the due date |
| 15 | Freeze is household-level; no new invoices while frozen; unfreeze only by admin after full payment (FR-04, SRS-09 §6) | `billing.overdue {Freeze, AdminManual, skip_issuing_while_frozen:true}` | as stated | |
| 16 | Manual verification of transfer proofs (FR-15) | Billing payment flow (BIL-20..24) | — | |
| 17 | Payment reminder on the 5th by WhatsApp with email backup (addendum §1) | `billing.reminders_days_from_due=[-5]`, `notify.routing` | [-5, 0] | |
| 18 | Leave max 1 month with admin fee; more = resign and re-register (addendum §3) vs "60 days/year" (PRODUCT.md) | `enrollment.leave` | max 30, fee 0 (owner sets), over_limit EndAndRequireReregistration | **Contradiction.** Resolved in favour of the addendum |
| 19 | Age minimum 3.5 y (piano, drums, vocal), 7 y (others) (addendum §9) | Catalog: `offerings.min_age_months` (42 / 84) + `enrollment.age.enforcement=Block` | in preset catalog | |
| 20 | Levels per instrument (addendum §6) | Catalog: `level_tracks` + `levels` | in preset catalog | |
| 21 | Class types private/intensif 2/3/semi 4–6/expatriate/tunas/holiday package (addendum §7) | Catalog: `offerings.capacity_min/max` + `price_plans` per offering variant | in preset catalog (sample subset) | "expatriate" = English-taught variant, modelled as a separate offering variant with its own price |
| 22 | Prices per instrument/type/level, registration fee (addendum §8) | Catalog: `price_plans` (effective-dated) | **Not shipped**. Real prices are tenant data. The preset ships illustrative round numbers marked `sample: true` | |
| 23 | Instructor check-in: GPS 150 m (+15 m jitter), window −30/+15 min, photo ≥640×480 ≤2 MB, face ≥30%, failures → pending review, SLA 2 h, escalate (SRS-04) | `attendance.instructor_checkin`, `attendance.review` | enabled, all as stated, face enabled | Studio coordinates are on `locations.geo`, set in the wizard |
| 24 | Check-in immutable after submit, one per session (SRS-04 §5) | Not a policy: DB constraint + no update route (ATT-04) | — | |
| 25 | Exam eligibility: active, attendance ≥90%, ≤1 no-show per rolling 3 months, <3 months tenure = incomplete (FR-07) | `enrollment.eligibility` | enabled, label "Exam eligibility", 3/9000/1/3 | |
| 26 | Notifications: critical email→WA, non-critical WA→email, 3 retries 10/20/40 s, breaker 5 failures/30 min, all to guardian, aggregated (FR-06) | `notify.*` | as stated | |
| 27 | Session reminder 1 h before (FR-06) | `notify.session_reminders_minutes_before` | [60] | |
| 28 | Session timeout owner/admin/instructor 30 min, guardian 60 min (FR-01) | `identity.idle_timeout_minutes` | 30/30/30/60/60 | |
| 29 | Lockout 5 failures / 15 min → lock 15 min + admin email (FR-01) | `identity.lockout` | as stated | |
| 30 | Temp password 8 chars, 7-day expiry, forced change (FR-01) | `identity.invite_expires_days=7`. **Changed:** invite *links* replace emailed temporary passwords (SEC-12) | 7 | Emailing passwords is insecure |
| 31 | Reset token 1 h, max 3/h (addendum §17) | `identity.reset_*` | 60 / 3 | |
| 32 | Password ≥8 alphanumeric (PROJECT_RULES) | `identity.password_min_length` | 8 | Generic default 10 |
| 33 | Audit 7-year retention, PII anonymised after 2 years, admin sees operational only (SRS-07) | `audit.*` | 7 / 2 / [Operational] | |
| 34 | Deletion requests ≤14 business days, blocked by open invoices (SRS-07 §4) | `core.privacy` | 14 / true | |
| 35 | Studio rental rehearsal (self, 1–4 h, 14 days ahead, cancel by H-1) and recording (admin approval, 30 days) (SRS-08) vs "remove studio rental" (addendum §4) | `rental` module + `booking_types` | Music preset: **rental disabled**. The `studio_rental` preset ships both types | Resolved: per-school choice, which is exactly what modularity is for |
| 36 | Instructor feedback categories progress/behaviour/attendance/recommendation 1–5; guardian rates own child's instructors 1–5 (addendum §13–14) | `feedback.*` | as stated | |
| 37 | Holiday packages, "Tunas" toddler class (addendum §7) | Catalog offerings with `kind=course`, `term=fixed_dates` | sample only | |
| 38 | Profile photo mandatory with initials fallback (addendum §12) | **Changed:** optional, initials avatar default | — | Mandatory photos of minors are a privacy risk with no benefit |
| 39 | WIB timezone hard-coded; Rupiah; Bahasa Indonesia | `core.timezone/currency/locale` | Asia/Jakarta / IDR / id | |
| 40 | Owner dashboard: revenue 6-month trend, attendance, frozen accounts, utilisation, audit summary (FR-13) | `reports.widgets` | all | |

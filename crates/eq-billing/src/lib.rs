//! Invoices, payments and billing policy for Ritma (spec/02 §2).
#![deny(clippy::float_arithmetic)]

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

static ADMIN_ONLY: &[ActorRole] = &[ActorRole::Admin];
static OWNER_ONLY: &[ActorRole] = &[ActorRole::Owner];
static OWNER_ADMIN_GUARDIAN_STUDENT: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Guardian,
    ActorRole::Student,
];
static ADMIN_GUARDIAN_STUDENT: &[ActorRole] = &[ActorRole::Admin, ActorRole::Guardian, ActorRole::Student];
static BILLING_PERMISSIONS: [PermissionDef; 7] = [
    PermissionDef {
        key: "billing.invoice.read",
        roles: OWNER_ADMIN_GUARDIAN_STUDENT,
    },
    PermissionDef {
        key: "billing.invoice.write",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "billing.payment.record",
        roles: ADMIN_GUARDIAN_STUDENT,
    },
    PermissionDef {
        key: "billing.payment.verify",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "billing.payer.unfreeze",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "billing.override.unfreeze",
        roles: OWNER_ONLY,
    },
    PermissionDef {
        key: "billing.override.cancel",
        roles: OWNER_ONLY,
    },
];

eq_core::impl_module_stub!(
    BillingModule,
    Billing,
    "billing",
    "receipt",
    false,
    &[],
    &[
        eq_core::module::ModuleId::Enrollment,
        eq_core::module::ModuleId::Rental
    ],
    &BILLING_PERMISSIONS
);

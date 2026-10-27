//! Invoices, payments and billing policy for Ritma (spec/02 §2).
#![deny(clippy::float_arithmetic)]

use eq_audit as _;
use eq_core as _;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

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
    ]
);

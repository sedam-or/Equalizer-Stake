//! Student enrollment and lifecycle management for Ritma (spec/02 §2).

use eq_audit as _;
use eq_core as _;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

eq_core::impl_module_stub!(
    EnrollmentModule,
    Enrollment,
    "enrollment",
    "user-round-plus",
    false,
    &[eq_core::module::ModuleId::Scheduling],
    &[]
);

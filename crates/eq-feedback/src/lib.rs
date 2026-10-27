//! Student progress notes and teacher feedback for Ritma (spec/02 §2).

use eq_audit as _;
use eq_core as _;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

eq_core::impl_module_stub!(
    FeedbackModule,
    Feedback,
    "feedback",
    "message-square-heart",
    false,
    &[eq_core::module::ModuleId::Enrollment],
    &[]
);

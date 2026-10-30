//! Student progress notes and teacher feedback for Ritma (spec/02 §2).

pub mod errors;

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

static ALL_ROLES: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Instructor,
    ActorRole::Guardian,
    ActorRole::Student,
];
static INSTRUCTOR_ONLY: &[ActorRole] = &[ActorRole::Instructor];
static ADMIN_ONLY: &[ActorRole] = &[ActorRole::Admin];
static GUARDIAN_STUDENT: &[ActorRole] = &[ActorRole::Guardian, ActorRole::Student];
static FEEDBACK_PERMISSIONS: [PermissionDef; 5] = [
    PermissionDef {
        key: "feedback.read",
        roles: ALL_ROLES,
    },
    PermissionDef {
        key: "feedback.notes.write",
        roles: INSTRUCTOR_ONLY,
    },
    PermissionDef {
        key: "feedback.notes.publish",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "feedback.ratings.write",
        roles: GUARDIAN_STUDENT,
    },
    PermissionDef {
        key: "feedback.ratings.moderate",
        roles: ADMIN_ONLY,
    },
];

eq_core::impl_module_stub!(
    FeedbackModule,
    Feedback,
    "feedback",
    "message-square-heart",
    false,
    &[eq_core::module::ModuleId::Enrollment],
    &[],
    &FEEDBACK_PERMISSIONS
);

//! Attendance check-in and review for Ritma (spec/02 §2).

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

static OWNER_ADMIN: &[ActorRole] = &[ActorRole::Owner, ActorRole::Admin];
static ADMIN_ONLY: &[ActorRole] = &[ActorRole::Admin];
static INSTRUCTOR_ONLY: &[ActorRole] = &[ActorRole::Instructor];
static ADMIN_INSTRUCTOR: &[ActorRole] = &[ActorRole::Admin, ActorRole::Instructor];
static ALL_ROLES: &[ActorRole] = &[
    ActorRole::Owner,
    ActorRole::Admin,
    ActorRole::Instructor,
    ActorRole::Guardian,
    ActorRole::Student,
];
static ATTENDANCE_PERMISSIONS: [PermissionDef; 5] = [
    PermissionDef {
        key: "attendance.checkin.submit",
        roles: INSTRUCTOR_ONLY,
    },
    PermissionDef {
        key: "attendance.review",
        roles: OWNER_ADMIN,
    },
    PermissionDef {
        key: "attendance.marks.write",
        roles: ADMIN_INSTRUCTOR,
    },
    PermissionDef {
        key: "attendance.marks.override",
        roles: ADMIN_ONLY,
    },
    PermissionDef {
        key: "attendance.marks.read",
        roles: ALL_ROLES,
    },
];

eq_core::impl_module_stub!(
    AttendanceModule,
    Attendance,
    "attendance",
    "clipboard-check",
    false,
    &[eq_core::module::ModuleId::Scheduling],
    &[],
    &ATTENDANCE_PERMISSIONS
);

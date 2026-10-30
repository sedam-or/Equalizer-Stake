//! Outbound messages and notification delivery for Ritma (spec/02 §2).

pub mod errors;

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

static OWNER_ADMIN: &[ActorRole] = &[ActorRole::Owner, ActorRole::Admin];
static NOTIFY_PERMISSIONS: [PermissionDef; 2] = [
    PermissionDef {
        key: "notify.channels.manage",
        roles: OWNER_ADMIN,
    },
    PermissionDef {
        key: "notify.log.read",
        roles: OWNER_ADMIN,
    },
];

eq_core::impl_module_stub!(
    NotifyModule,
    Notify,
    "notify",
    "bell",
    false,
    &[],
    &[],
    &NOTIFY_PERMISSIONS
);

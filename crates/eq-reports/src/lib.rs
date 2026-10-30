//! Owner dashboards and school reports for Ritma (spec/02 §2).

use eq_audit as _;
use eq_core as _;
use eq_core::actor::ActorRole;
use eq_core::module::PermissionDef;
use eq_identity as _;
use eq_scheduling as _;
use serde_json as _;
use utoipa_axum as _;

static OWNER_ONLY: &[ActorRole] = &[ActorRole::Owner];
static REPORTS_PERMISSIONS: [PermissionDef; 1] = [PermissionDef {
    key: "reports.revenue.read",
    roles: OWNER_ONLY,
}];

eq_core::impl_module_stub!(
    ReportsModule,
    Reports,
    "reports",
    "chart-no-axes-column",
    false,
    &[],
    &[],
    &REPORTS_PERMISSIONS
);

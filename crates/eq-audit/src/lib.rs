//! The audit log module for Ritma (spec/02 §2).

extern crate self as eq_audit;

pub mod errors;
pub use eq_audit_derive::Auditable;
use eq_core as _;
mod http;
pub mod jobs;
pub mod module;
pub mod policy;
mod repo;
pub use http::AuditPolicyProvider;
pub use policy::Category;
pub mod record;
pub use record::{AuditEvent, record, record_once_for_actor_entity_window};
pub mod sealer;
pub use jobs::{AuditRetentionJob, AuditVerifyRecentJob};
pub use module::AuditModule;
pub use sealer::{Sealer, VerifyReport};
pub mod __private {
    pub use serde;
    pub use serde_json;
}
use serde_json::Value;
use utoipa_axum as _;

/// AUD-02: serializable snapshot of the fields safe to include in an audit entry.
pub trait Auditable {
    /// Returns a JSON snapshot with fields marked `#[audit(redact)]` replaced.
    fn audit_snapshot(&self) -> Value;
}

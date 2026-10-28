//! The audit log module for Ritma (spec/02 §2).

extern crate self as eq_audit;

pub use eq_audit_derive::Auditable;
use eq_core as _;
pub mod policy;
mod repo;
pub use policy::Category;
pub mod record;
pub use record::{AuditEvent, record};
pub mod __private {
    pub use serde;
    pub use serde_json;
}
use serde_json::Value;
use utoipa_axum as _;

eq_core::impl_module_stub!(AuditModule, Audit, "audit", "scroll-text", true, &[], &[]);

/// AUD-02: serializable snapshot of the fields safe to include in an audit entry.
pub trait Auditable {
    /// Returns a JSON snapshot with fields marked `#[audit(redact)]` replaced.
    fn audit_snapshot(&self) -> Value;
}

//! Request and job identity passed to transaction helpers (ARC-04, SEC-05).

use crate::ids::PersonId;
use std::net::IpAddr;
use uuid::Uuid;

/// ARC-04: role associated with an acting person or system process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorRole {
    Anonymous,
    Owner,
    Admin,
    Instructor,
    Guardian,
    Student,
    System,
    Platform,
}

/// ARC-04, SEC-05: minimal actor facts propagated to a tenant transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorCtx {
    pub person_id: Option<PersonId>,
    pub role: ActorRole,
    pub ip: Option<IpAddr>,
    pub request_id: Option<Uuid>,
}

impl ActorCtx {
    /// ARC-04: builds a personless request actor before authentication.
    pub fn anonymous(ip: Option<IpAddr>, request_id: Option<Uuid>) -> Self {
        Self {
            person_id: None,
            role: ActorRole::Anonymous,
            ip,
            request_id,
        }
    }

    /// ARC-04: builds a personless actor for a named system process.
    pub fn system(_kind: &'static str) -> Self {
        Self {
            person_id: None,
            role: ActorRole::System,
            ip: None,
            request_id: None,
        }
    }
}

//! Strongly typed identifiers shared across core and feature modules (CQ-21).

use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

/// DOM-01: identifier for a tenant aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TenantId(pub Uuid);

/// DOM-01: identifier for a person aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct PersonId(pub Uuid);

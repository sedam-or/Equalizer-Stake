//! DOM-01: Rust rows for the core tables introduced by T-101.

use crate::ids::{FileId, LocationId, PersonId, ResourceId, TenantId};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::{FromRow, Type, postgres::types::PgRange};
use std::ops::Bound;
use uuid::Uuid;

/// DOM-01: tenant row, including denormalised core policy defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: TenantId,
    pub slug: String,
    pub name: String,
    pub country: String,
    pub preset_id: String,
    pub preset_version: i32,
    pub setup_step: i32,
    pub setup_completed_at: Option<DateTime<Utc>>,
    pub status: TenantStatus,
    pub locale: String,
    pub currency: String,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
}

/// DOM-01: status values stored by `tenants.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    Closed,
}

/// DOM-01: location row. Coordinates are paired by a database check constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Location {
    pub id: LocationId,
    pub tenant_id: TenantId,
    pub name: String,
    pub address: Option<String>,
    pub timezone: Option<String>,
    pub geo_lat: Option<f64>,
    pub geo_lng: Option<f64>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// DOM-01: bookable resource row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: ResourceId,
    pub tenant_id: TenantId,
    pub location_id: LocationId,
    pub name: String,
    pub kind: ResourceKind,
    pub capacity: i32,
    pub rentable: bool,
    pub color: Option<String>,
    pub sort: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// DOM-01: resource kind values stored by `resources.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ResourceKind {
    Room,
    Equipment,
    Other,
}

/// DOM-01: person row containing encrypted fields only; decrypted values stay in service DTOs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Person {
    pub id: PersonId,
    pub tenant_id: TenantId,
    pub display_name: String,
    pub legal_name_enc: Option<Vec<u8>>,
    pub email_enc: Option<Vec<u8>>,
    pub email_bidx: Option<Vec<u8>>,
    pub phone_enc: Option<Vec<u8>>,
    pub phone_bidx: Option<Vec<u8>>,
    pub birth_date_enc: Option<Vec<u8>>,
    pub address_enc: Option<Vec<u8>>,
    pub locale: Option<String>,
    pub avatar_file_id: Option<FileId>,
    pub anonymized_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

/// DOM-01: tenant membership row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Membership {
    pub tenant_id: TenantId,
    pub person_id: PersonId,
    pub role: Role,
    pub status: MembershipStatus,
    pub created_at: DateTime<Utc>,
}

/// DOM-01: roles stored by `memberships.role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Role {
    Owner,
    Admin,
    Instructor,
    Guardian,
    Student,
}

/// DOM-01: membership status values stored by `memberships.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum MembershipStatus {
    Invited,
    Active,
    Deactivated,
}

/// DOM-01: guardian-to-student link row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct PersonLink {
    pub tenant_id: TenantId,
    pub guardian_id: PersonId,
    pub student_id: PersonId,
    pub relationship: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

/// DOM-01: school closure over a date range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Closure {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub location_id: Option<LocationId>,
    #[serde(with = "pg_date_range_serde")]
    pub dates: PgRange<NaiveDate>,
    pub name: serde_json::Value,
    pub kind: ClosureKind,
    pub created_at: DateTime<Utc>,
}

mod pg_date_range_serde {
    use super::{Bound, Deserialize, Deserializer, NaiveDate, PgRange, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct RangeRepr {
        start: BoundRepr,
        end: BoundRepr,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "kind", content = "date", rename_all = "snake_case")]
    enum BoundRepr {
        Included(NaiveDate),
        Excluded(NaiveDate),
        Unbounded,
    }

    impl From<Bound<NaiveDate>> for BoundRepr {
        fn from(bound: Bound<NaiveDate>) -> Self {
            match bound {
                Bound::Included(date) => Self::Included(date),
                Bound::Excluded(date) => Self::Excluded(date),
                Bound::Unbounded => Self::Unbounded,
            }
        }
    }

    impl From<BoundRepr> for Bound<NaiveDate> {
        fn from(bound: BoundRepr) -> Self {
            match bound {
                BoundRepr::Included(date) => Self::Included(date),
                BoundRepr::Excluded(date) => Self::Excluded(date),
                BoundRepr::Unbounded => Self::Unbounded,
            }
        }
    }

    pub(super) fn serialize<S>(range: &PgRange<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RangeRepr {
            start: range.start.into(),
            end: range.end.into(),
        }
        .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<PgRange<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let range = RangeRepr::deserialize(deserializer)?;
        Ok(PgRange {
            start: range.start.into(),
            end: range.end.into(),
        })
    }
}

/// DOM-01: closure kind values stored by `closures.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ClosureKind {
    Holiday,
    Closure,
}

/// DOM-01: stored file metadata. File contents are not loaded into row structs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct FileRow {
    pub id: FileId,
    pub tenant_id: TenantId,
    pub purpose: FilePurpose,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: Vec<u8>,
    pub storage_key: String,
    pub encrypted: bool,
    pub created_by: Option<PersonId>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// DOM-01: file-purpose values stored by `files.purpose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum FilePurpose {
    Avatar,
    CheckinPhoto,
    PaymentProof,
    RegistrationDoc,
    Export,
    Import,
    Logo,
}

/// DOM-01: admin attention queue row. `ref_id` is a logical reference, not a cross-module FK.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct AttentionItem {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub module_id: String,
    pub kind: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub severity: Severity,
    pub dedupe_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<PersonId>,
}

/// DOM-01: attention severity values stored by `attention_items.severity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Severity {
    Low,
    Normal,
    High,
}

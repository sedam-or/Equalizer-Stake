//! Typed synchronous ports used for cross-module queries (ARC-08).

pub mod sinks;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::db::Tx;
use crate::error::AppError;
use crate::ids::TenantId;
use async_trait::async_trait;

/// ARC-08: stores implementations by their requested port type.
#[derive(Default)]
pub struct PortRegistry {
    entries: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

/// POL-12: asks the owning Billing module whether tenant invoices lock currency changes.
#[async_trait]
pub trait CurrencyLockCheck: Send + Sync {
    /// Returns true when the tenant has an invoice and its currency cannot change.
    async fn is_locked(&self, _tx: &mut Tx, _tenant: TenantId) -> Result<bool, AppError> {
        Ok(false)
    }
}

/// POL-12: default Core behavior before Billing registers its invoice-aware adapter.
#[derive(Debug, Default)]
pub struct UnlockedCurrencyLockCheck;

#[async_trait]
impl CurrencyLockCheck for UnlockedCurrencyLockCheck {}

impl fmt::Debug for PortRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PortRegistry")
            .field("registered_ports", &self.entries.len())
            .finish()
    }
}

impl PortRegistry {
    /// ARC-08: registers a typed port implementation.
    pub fn register<P>(&mut self, port: Arc<P>)
    where
        P: ?Sized + Send + Sync + 'static,
    {
        self.entries.insert(TypeId::of::<P>(), Box::new(port));
    }

    /// ARC-08: returns a typed implementation when its owning module is compiled and enabled.
    pub fn get<P>(&self) -> Option<Arc<P>>
    where
        P: ?Sized + Send + Sync + 'static,
    {
        self.entries
            .get(&TypeId::of::<P>())
            .and_then(|entry| entry.downcast_ref::<Arc<P>>())
            .cloned()
    }
}

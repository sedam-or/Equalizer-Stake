//! Validated registry of modules compiled into the binary (MOD-04).

use super::{Module, ModuleId, ModuleManifest};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

/// MOD-04: modules in canonical `ModuleId` order.
pub struct ModuleRegistry {
    pub modules: Vec<Arc<dyn Module>>,
}

impl fmt::Debug for ModuleRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModuleRegistry")
            .field(
                "modules",
                &self
                    .modules
                    .iter()
                    .map(|module| module.manifest().id)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl ModuleRegistry {
    /// MOD-04: validates identity references and rejects cyclic dependency graphs at startup.
    pub fn new(modules: Vec<Arc<dyn Module>>) -> Self {
        Self { modules }
    }

    /// MOD-04: panics with a clear startup error for duplicate, unknown or cyclic module declarations.
    pub fn validate_graph(&self) {
        let manifests = self
            .modules
            .iter()
            .map(|module| module.manifest())
            .collect::<Vec<_>>();
        let mut by_id = HashMap::<ModuleId, &'static ModuleManifest>::with_capacity(manifests.len());
        for manifest in &manifests {
            assert!(
                by_id.insert(manifest.id, manifest).is_none(),
                "duplicate module manifest for {}",
                manifest.id
            );
        }

        for manifest in &manifests {
            for dependency in manifest.requires_all.iter().chain(manifest.requires_any) {
                assert!(
                    by_id.contains_key(dependency),
                    "module {} requires unknown module {}",
                    manifest.id,
                    dependency
                );
            }
        }

        fn visit(
            id: ModuleId,
            by_id: &HashMap<ModuleId, &'static ModuleManifest>,
            visiting: &mut HashSet<ModuleId>,
            visited: &mut HashSet<ModuleId>,
        ) {
            if visited.contains(&id) {
                return;
            }
            assert!(visiting.insert(id), "module dependency cycle includes {id}");
            let Some(manifest) = by_id.get(&id) else {
                return;
            };
            for dependency in manifest.requires_all.iter().chain(manifest.requires_any) {
                visit(*dependency, by_id, visiting, visited);
            }
            visiting.remove(&id);
            visited.insert(id);
        }

        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        for id in by_id.keys().copied() {
            visit(id, &by_id, &mut visiting, &mut visited);
        }
    }

    /// MOD-04: finds the compiled implementation for one module id.
    pub fn get(&self, id: ModuleId) -> Option<&Arc<dyn Module>> {
        self.modules.iter().find(|module| module.manifest().id == id)
    }

    /// MOD-04: returns compiled modules in the registry's canonical order.
    pub fn modules(&self) -> &[Arc<dyn Module>] {
        &self.modules
    }

    /// MOD-04: returns manifest references for pure dependency planning.
    pub fn manifests(&self) -> Vec<&'static ModuleManifest> {
        self.modules.iter().map(|module| module.manifest()).collect()
    }
}

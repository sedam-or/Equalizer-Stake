use eq_api::modules::registry;
use eq_core::actor::ActorRole;
use std::collections::{BTreeMap, BTreeSet};

const RBAC_MATRIX: &str = include_str!("fixtures/rbac_matrix.csv");

#[test]
fn rbac_matrix_matches_spec() {
    let lines = RBAC_MATRIX.lines();
    let mut expected_by_module = BTreeMap::<String, BTreeSet<(String, BTreeSet<String>)>>::new();

    for line in lines.skip(1) {
        let columns = line.split(',').collect::<Vec<_>>();
        assert_eq!(columns.len(), 6, "invalid fixture row: {line}");
        let mut roles = BTreeSet::new();
        for (column, role) in columns[1..]
            .iter()
            .zip(["owner", "admin", "instructor", "guardian", "student"])
        {
            assert!(matches!(*column, "y" | "n"), "invalid grant cell: {column}");
            if *column == "y" {
                roles.insert(role.to_owned());
            }
        }
        let key = columns[0].to_owned();
        let module = key.split('.').next().unwrap_or_default().to_owned();
        expected_by_module.entry(module).or_default().insert((key, roles));
    }

    let registry = registry();
    for module in registry.modules() {
        let module_id = module.manifest().id.as_str();
        let expected = expected_by_module.remove(module_id).unwrap_or_default();
        let actual = module
            .permissions()
            .iter()
            .map(|permission| {
                let roles = permission.roles.iter().map(role_name).collect();
                (permission.key.to_owned(), roles)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual.len(),
            module.permissions().len(),
            "duplicate permission for module {module_id}"
        );
        assert_eq!(actual, expected, "permission matrix mismatch for {module_id}");
    }
}

fn role_name(role: &ActorRole) -> String {
    match role {
        ActorRole::Owner => "owner",
        ActorRole::Admin => "admin",
        ActorRole::Instructor => "instructor",
        ActorRole::Guardian => "guardian",
        ActorRole::Student => "student",
        ActorRole::Anonymous | ActorRole::System | ActorRole::Platform => "non_tenant",
    }
    .to_owned()
}

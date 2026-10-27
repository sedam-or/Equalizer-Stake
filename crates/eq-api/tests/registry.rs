use eq_api::modules::registry;
use eq_core::module::ModuleId;

#[test]
fn registry_graph_valid() {
    registry().validate_graph();
}

#[test]
fn registry_order_matches_enum() {
    let mut expected = vec![
        ModuleId::Core,
        ModuleId::Identity,
        ModuleId::Audit,
        ModuleId::Scheduling,
    ];
    #[cfg(feature = "enrollment")]
    expected.push(ModuleId::Enrollment);
    #[cfg(feature = "billing")]
    expected.push(ModuleId::Billing);
    #[cfg(feature = "attendance")]
    expected.push(ModuleId::Attendance);
    #[cfg(feature = "rental")]
    expected.push(ModuleId::Rental);
    #[cfg(feature = "notify")]
    expected.push(ModuleId::Notify);
    #[cfg(feature = "reports")]
    expected.push(ModuleId::Reports);
    #[cfg(feature = "feedback")]
    expected.push(ModuleId::Feedback);

    let actual = registry()
        .modules()
        .iter()
        .map(|module| module.manifest().id)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

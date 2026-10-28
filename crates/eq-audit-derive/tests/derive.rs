use eq_audit::Auditable;

#[derive(Auditable)]
struct Snapshot {
    display_name: String,
    #[audit(redact)]
    password: String,
    #[audit(skip)]
    internal_marker: String,
}

#[derive(Auditable)]
struct TupleSnapshot(String, #[audit(redact)] String, #[audit(skip)] String);

#[test]
fn redact_attr() {
    let snapshot = Snapshot {
        display_name: "fake learner".to_owned(),
        password: "test-secret-audit-field".to_owned(),
        internal_marker: "test-secret-skipped-field".to_owned(),
    };

    let value = snapshot.audit_snapshot();
    assert_eq!(value["password"], "[redacted]");
    assert_eq!(value["display_name"], "fake learner");
    assert_eq!(snapshot.password, "test-secret-audit-field");
    assert!(!value.to_string().contains("test-secret-audit-field"));
}

#[test]
fn skip_attr() {
    let snapshot = Snapshot {
        display_name: "fake learner".to_owned(),
        password: "test-secret-audit-field".to_owned(),
        internal_marker: "test-secret-skipped-field".to_owned(),
    };
    let tuple = TupleSnapshot(
        "kept".to_owned(),
        "test-secret-tuple-field".to_owned(),
        "skipped".to_owned(),
    );

    assert!(snapshot.audit_snapshot().get("internal_marker").is_none());
    assert_eq!(snapshot.internal_marker, "test-secret-skipped-field");
    assert_eq!(tuple.audit_snapshot()["0"], "kept");
    assert_eq!(tuple.audit_snapshot()["1"], "[redacted]");
    assert_eq!(tuple.1, "test-secret-tuple-field");
    assert!(tuple.audit_snapshot().get("2").is_none());
    assert_eq!(tuple.2, "skipped");
    assert!(
        !tuple
            .audit_snapshot()
            .to_string()
            .contains("test-secret-tuple-field")
    );
}

use std::process::Command;

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_ritma"))
        .arg("--version")
        .output()
        .expect("ritma --version should run");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"ritma 0.1.0\n");
}

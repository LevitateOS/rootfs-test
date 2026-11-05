//! Exercise the real runtime-check CLI without needing a VM or root privileges.
use std::process::Command;

#[test]
fn invalid_category_fails_before_artifact_access() {
    let fixture = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rootfs-tests"))
        .args(["run", "--category", "typo-category", "--tarball"])
        .arg(fixture.path().join("missing.tar.xz"))
        .current_dir(fixture.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no runtime checks selected") && stderr.contains("available categories"),
        "{stderr}"
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Extracting"));
    assert_eq!(
        std::fs::read_dir(fixture.path()).unwrap().count(),
        0,
        "invalid selection mutated the working directory"
    );
}

#[test]
fn rootfs_source_requires_a_directory_and_preserves_the_input() {
    let fixture = tempfile::tempdir().unwrap();
    let source = fixture.path().join("not-a-directory");
    std::fs::write(&source, "preserve me").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rootfs-tests"))
        .args(["run", "--rootfs"])
        .arg(&source)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected an existing rootfs directory")
    );
    assert_eq!(std::fs::read_to_string(source).unwrap(), "preserve me");
}

#[test]
fn rootfs_and_tarball_sources_are_mutually_exclusive() {
    let output = Command::new(env!("CARGO_BIN_EXE_rootfs-tests"))
        .args([
            "run",
            "--rootfs",
            "/unused-rootfs",
            "--tarball",
            "/unused.tar.xz",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
}

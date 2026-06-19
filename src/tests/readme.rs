use std::process::Command;

#[test]
fn readme_getting_started_snippet_must_compile() {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
        .args(["check", "--example", "readme_getting_started"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo check should run");

    assert!(
        output.status.success(),
        "README getting-started snippet must compile.\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

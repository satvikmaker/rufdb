use std::process::Command;
use tempfile::TempDir;

fn rufdb_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rufdb"))
}

#[test]
fn cli_init_creates_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("cli.json");

    let output = rufdb_cmd()
        .args([
            path.to_str().unwrap(),
            "init",
            r#"{"posts":[],"count":0}"#,
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(path.exists());

    let content = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(value["posts"], serde_json::json!([]));
    assert_eq!(value["count"], serde_json::json!(0));
}

#[test]
fn cli_get_reads_whole_db() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("get.json");

    // Init first
    rufdb_cmd()
        .args([path.to_str().unwrap(), "init", r#"{"name":"test"}"#])
        .output()
        .unwrap();

    let output = rufdb_cmd()
        .args([path.to_str().unwrap(), "get"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("test"));
}

#[test]
fn cli_get_nested_key() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested.json");

    rufdb_cmd()
        .args([
            path.to_str().unwrap(),
            "init",
            r#"{"user":{"name":"Alice","age":30}}"#,
        ])
        .output()
        .unwrap();

    let output = rufdb_cmd()
        .args([path.to_str().unwrap(), "get", "user.name"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Alice"));
}

#[test]
fn cli_set_value() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("set.json");

    rufdb_cmd()
        .args([path.to_str().unwrap(), "init", r#"{"count":0}"#])
        .output()
        .unwrap();

    let output = rufdb_cmd()
        .args([path.to_str().unwrap(), "set", "count", "42"])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify
    let content = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(value["count"], serde_json::json!(42));
}

#[test]
fn cli_delete_key() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("delete.json");

    rufdb_cmd()
        .args([
            path.to_str().unwrap(),
            "init",
            r#"{"a":1,"b":2}"#,
        ])
        .output()
        .unwrap();

    let output = rufdb_cmd()
        .args([path.to_str().unwrap(), "delete", "b"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let content = std::fs::read_to_string(&path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(value.get("b"), None);
    assert_eq!(value["a"], serde_json::json!(1));
}

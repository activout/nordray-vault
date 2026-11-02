use base64::Engine;
use std::process::{Command, Output};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn keyfile(name: &str) -> String {
    let bytes = std::fs::read(format!("{FIXTURES}/{name}.keyx")).unwrap();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn run(kdbx: &str, key: &str, extra: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_nordray-vault"));
    cmd.env_clear()
        .env("NORDRAY_VAULT_FILE", format!("{FIXTURES}/{kdbx}.kdbx"))
        .env("NORDRAY_VAULT_KEYFILE", keyfile(key))
        .env("NORDRAY_VAULT_STORAGE_PASSWORD", "unused")
        .args(["--", "/usr/bin/env"]);
    for (name, value) in extra {
        cmd.env(name, value);
    }
    cmd.output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn exports_root_entries_and_hides_its_own_variables() {
    let out = run("test", "test", &[]);
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(stdout.contains("FOO=bar\n"));
    assert!(stdout.contains("MyApp__Mail__ApiKey=s3cret value\n"));
    assert!(!stdout.contains("IGNORED"), "subgroups are ignored");
    assert!(!stdout.contains("NORDRAY_VAULT_"));
}

#[test]
fn environment_wins_and_names_it() {
    let out = run("test", "test", &[("FOO", "override")]);
    assert!(out.status.success());
    assert!(text(&out.stdout).contains("FOO=override\n"));
    let stderr = text(&out.stderr);
    assert!(stderr.contains("FOO is already set"));
    assert!(!stderr.contains("bar"));
}

#[test]
fn invalid_title_fails() {
    let out = run("invalid", "invalid", &[]);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("not a valid environment variable name"));
    assert!(out.stdout.is_empty());
}

#[test]
fn wrong_key_fails_without_secrets() {
    let out = run("test", "other", &[]);
    assert!(!out.status.success());
    let stderr = text(&out.stderr);
    assert!(stderr.contains("cannot open the KeePass file"));
    assert!(!stderr.contains("bar") && !stderr.contains("s3cret"));
    assert!(out.stdout.is_empty());
}

#[test]
fn missing_command_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_nordray-vault")).output().unwrap();
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("usage"));
}

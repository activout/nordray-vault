//! nordray-vault: unlock an app's KeePass file, export its entries as environment variables, exec the app.
//!
//! Usage: nordray-vault -- <command> [args…]. Error messages never contain a value, key or password.

use base64::Engine;
use keepass::{Database, DatabaseKey};
use std::env;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

const PREFIX: &str = "NORDRAY_VAULT_";
const DEFAULT_HOST: &str = "storage.bunnycdn.com"; // Bunny's main (Falkenstein) endpoint

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = match args.split_first() {
        Some((dashes, command)) if dashes == "--" && !command.is_empty() => command,
        _ => return fail("usage: nordray-vault -- <command> [args…]"),
    };
    let entries = match load() {
        Ok(entries) => entries,
        Err(message) => return fail(&message),
    };

    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..]);
    for (name, _) in env::vars_os() {
        if name.to_string_lossy().starts_with(PREFIX) {
            cmd.env_remove(name);
        }
    }
    for (name, value) in entries {
        if env::var_os(&name).is_some() {
            eprintln!("nordray-vault: {name} is already set; keeping the environment's value");
        } else {
            cmd.env(name, value);
        }
    }
    let error = cmd.exec();
    fail(&format!("cannot run {}: {error}", command[0]))
}

fn fail(message: &str) -> ExitCode {
    eprintln!("nordray-vault: {message}");
    ExitCode::FAILURE
}

fn var(name: &str) -> Result<String, String> {
    env::var(format!("{PREFIX}{name}")).map_err(|_| format!("{PREFIX}{name} is not set"))
}

fn load() -> Result<Vec<(String, String)>, String> {
    let file = var("FILE")?;
    let keyfile = base64::engine::general_purpose::STANDARD
        .decode(var("KEYFILE")?.trim())
        .map_err(|_| format!("{PREFIX}KEYFILE is not valid base64"))?;
    let kdbx = if file.starts_with('/') {
        std::fs::read(&file).map_err(|e| format!("cannot read {file}: {e}"))?
    } else {
        let host = env::var(format!("{PREFIX}STORAGE_HOST")).unwrap_or_else(|_| DEFAULT_HOST.into());
        download(&storage_url(&host, &var("STORAGE_ZONE")?, &file), &var("STORAGE_PASSWORD")?)?
    };
    entries(&kdbx, &keyfile)
}

fn storage_url(host: &str, zone: &str, file: &str) -> String {
    format!("https://{host}/{zone}/{file}")
}

fn download(url: &str, password: &str) -> Result<Vec<u8>, String> {
    let mut response = ureq::get(url)
        .header("AccessKey", password)
        .call()
        .map_err(|e| format!("cannot download {url}: {e}"))?;
    response
        .body_mut()
        .read_to_vec()
        .map_err(|e| format!("cannot download {url}: {e}"))
}

/// The root group's entries: title = variable name, password = value. Subgroups are ignored.
fn entries(kdbx: &[u8], keyfile: &[u8]) -> Result<Vec<(String, String)>, String> {
    let key = DatabaseKey::new()
        .with_keyfile(&mut &keyfile[..])
        .map_err(|_| "the key file is not a valid KeePass key file".to_string())?;
    let db = Database::open(&mut &kdbx[..], key)
        .map_err(|e| format!("cannot open the KeePass file (wrong key file?): {e}"))?;
    let mut result = Vec::new();
    for entry in db.root().entries() {
        let name = entry.get_title().unwrap_or_default();
        if !valid_name(name) {
            return Err(format!("entry title {name:?} is not a valid environment variable name"));
        }
        result.push((name.to_string(), entry.get_password().unwrap_or_default().to_string()));
    }
    Ok(result)
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_storage_url() {
        assert_eq!(
            storage_url("storage.bunnycdn.com", "my-vault-zone", "my-app.kdbx"),
            "https://storage.bunnycdn.com/my-vault-zone/my-app.kdbx"
        );
    }

    #[test]
    fn validates_names() {
        assert!(valid_name("MyApp__Mail__ApiKey"));
        assert!(valid_name("_X1"));
        assert!(!valid_name(""));
        assert!(!valid_name("1X"));
        assert!(!valid_name("not valid"));
        assert!(!valid_name("A-B"));
    }
}

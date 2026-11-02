# nordray-vault

Secrets for containerised apps on [bunny.net](https://bunny.net) (Magic Containers), without running a vault server.

Each app has one KeePass file (`<app>.kdbx`) in a Bunny storage zone and holds only that file's key file. The
`nordray-vault` binary is the container's entrypoint. It downloads the app's file, unlocks it, exports the entries as
environment variables and execs the app, so the app itself needs no changes. Every app can download every file in
the zone, but each can decrypt only its own. Secrets are edited with KeePassXC or `keepassxc-cli`.

## Use in an app

```dockerfile
COPY --from=ghcr.io/activout/nordray-vault:v0.1.0 /nordray-vault /usr/local/bin/
ENTRYPOINT ["nordray-vault", "--", "dotnet", "MyApp.dll"]
```

| Variable | Meaning |
| --- | --- |
| `NORDRAY_VAULT_FILE` | The file in the zone, e.g. `my-app.kdbx`. An absolute path reads a local file instead |
| `NORDRAY_VAULT_KEYFILE` | The app's key file (`.keyx`), base64 |
| `NORDRAY_VAULT_STORAGE_ZONE` | The storage zone's name |
| `NORDRAY_VAULT_STORAGE_PASSWORD` | The zone's **read-only** password |
| `NORDRAY_VAULT_STORAGE_HOST` | The zone's storage endpoint. Defaults to `storage.bunnycdn.com` (Falkenstein); other regions use e.g. `se.storage.bunnycdn.com`, `ny.storage.bunnycdn.com` |

The zone's name is a little extra protection. Pick one that isn't easy to guess, since anyone with the read-only
password and the name can download the encrypted files.

Each entry in the root group becomes one variable: its title is the name and its password is the value. Subgroups,
including the Recycle Bin, are ignored. For example, the title `MyApp__Database__Password` becomes
`MyApp:Database:Password` in .NET configuration. A variable that is already set in the environment wins, and its
name is written to stderr. `NORDRAY_VAULT_*` variables are removed before the app starts. Any error stops the start-up
without printing a secret.

The binary is static (musl), so it runs in any Linux image. Releases also attach it as a download, with checksums
and a build provenance attestation.

## Managing secrets

The steps are written once, as an agent skill: [skills/nordray-vault/SKILL.md](skills/nordray-vault/SKILL.md). It
covers adding, changing and removing a secret, adding an app, rotating a key file and rolling back, using
`keepassxc-cli` and the [bunny CLI](https://www.npmjs.com/package/@bunny.net/cli). To use it with Claude Code:

```sh
ln -s "$PWD/skills/nordray-vault" ~/.claude/skills/nordray-vault
mkdir -p ~/.config/nordray-vault && chmod 700 ~/.config/nordray-vault
echo '<your zone name>' > ~/.config/nordray-vault/zone
```

## Build and test

```sh
docker build --target test .   # cargo test, as in CI
docker build -t nordray-vault .
```

The fixtures in `tests/fixtures` were made with `keepassxc-cli` 2.7 and hold dummy values only.

## License

MIT. See [LICENSE](LICENSE).

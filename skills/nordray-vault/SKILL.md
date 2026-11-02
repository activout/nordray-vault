---
name: nordray-vault
description: Add, change, remove or list an app's secrets in nordray-vault, create a vault for a new app, rotate its key file or roll back a change, using keepassxc-cli and the bunny CLI on the user's vault storage zone. Use when the user asks to set, update, rotate or look up a secret for one of their Bunny apps, or mentions nordray-vault or an app's .kdbx file.
---

# nordray-vault

## Model

- One KeePass file per app, `<app>.kdbx`, at the root of the user's vault storage zone. **The zone's name is not
  written in any repository:** read it with `ZONE=$(cat ~/.config/nordray-vault/zone)`. If that file is missing, ask
  the user for the name and offer to save it there. Copies made before each change live under `backups/`; apps never
  read them.
- Each file is unlocked by its **key file alone** (no password). Key files live at
  `~/.config/nordray-vault/<app>.keyx`, with a copy in the user's password manager. Never in the zone, in git, in a
  log or in the conversation.
- **Entry title = environment variable name** (`[A-Za-z_][A-Za-z0-9_]*`; a double underscore separates .NET
  configuration levels, e.g. `MyApp__Database__Password`). **Password = value.** Only entries in the root group are
  exported; subgroups (including the Recycle Bin) are ignored.
- The app's container starts as `nordray-vault -- <app command>` and has these env vars on the Bunny app:
  - `NORDRAY_VAULT_FILE=<app>.kdbx`;
  - `NORDRAY_VAULT_KEYFILE=<base64 of the key file>`;
  - `NORDRAY_VAULT_STORAGE_ZONE=<zone>`;
  - `NORDRAY_VAULT_STORAGE_PASSWORD=<the zone's read-only password>`;
  - `NORDRAY_VAULT_STORAGE_HOST`, if the zone isn't in Falkenstein (`bunny storage zones show $ZONE` shows it).

  A variable set directly on the Bunny app wins over the file, and the app logs its name.
- An app reads its file only at start-up: **a change takes effect when the app restarts**.

Tools:
- `keepassxc-cli`, from KeePassXC. On macOS, `brew install --cask keepassxc` puts it on the PATH.
- The `bunny` CLI. The bunny-cli skill has details for `bunny storage files`; the CLI finds the zone's host itself.
- The `nordray-vault` binary, for validation. Or copy it from the image `ghcr.io/activout/nordray-vault`.

## Rules for the agent

- **Never print a secret value.**
  - List names with `keepassxc-cli ls`.
  - Never run `keepassxc-cli show` with `-s`/`--show-protected` or `-a Password`.
  - Never `cat` a key file or the zone's passwords.
- Values never go on a command line, where they'd end up in shell history or the process list. When a value is
  needed, have the user type it: suggest the exact command with the `!` prefix, so `keepassxc-cli -p` prompts them
  and the value never enters the conversation.
- Work on a copy in a fresh temporary directory (`TMP=$(mktemp -d)`) and `rm -rf "$TMP"` afterwards.
- **Confirm with the user before** each upload to the zone, each change to a Bunny app's env vars and each restart:
  they change running apps.
- Never write the zone's name, an app's name or a secret into a public repository.

## Change, add or remove a secret

```sh
ZONE=$(cat ~/.config/nordray-vault/zone); APP=my-app; KEY=~/.config/nordray-vault/$APP.keyx; TMP=$(mktemp -d)
bunny storage files download $APP.kdbx --zone "$ZONE" --out "$TMP/$APP.kdbx"
bunny storage files upload "$TMP/$APP.kdbx" --zone "$ZONE" --to "backups/$APP-$(date -u +%Y%m%dT%H%M%SZ).kdbx"
keepassxc-cli ls --no-password -k "$KEY" "$TMP/$APP.kdbx"                       # names only
```

Then the user types one of these with `!`; each prompts for the value:

```sh
! keepassxc-cli add  --no-password -k "$KEY" -p "$TMP/$APP.kdbx" MyApp__Mail__ApiKey   # new
! keepassxc-cli edit --no-password -k "$KEY" -p "$TMP/$APP.kdbx" MyApp__Mail__ApiKey   # change
```

Removing needs no value, so the agent may run it: `keepassxc-cli rm --no-password -k "$KEY" "$TMP/$APP.kdbx" <NAME>`.
`rm` moves the entry to the Recycle Bin, which is a subgroup and therefore not exported.

Validate before uploading. This prints names only, and proves that the key file opens the file and every title is a
valid name:

```sh
NORDRAY_VAULT_FILE="$TMP/$APP.kdbx" NORDRAY_VAULT_KEYFILE="$(base64 -i "$KEY")" \
  nordray-vault -- sh -c 'env | cut -d= -f1 | sort'
```

Without a local binary, run the same in Docker, with `-v "$TMP:$TMP:ro"`, on an image that copies `/nordray-vault`
from `ghcr.io/activout/nordray-vault:<version>`.

After the user confirms, upload. Then restart the app, from the Bunny dashboard or through the Magic Containers API
with `bunny api`:

```sh
bunny storage files upload "$TMP/$APP.kdbx" --zone "$ZONE" --to $APP.kdbx
rm -rf "$TMP"
```

## New app

```sh
mkdir -p ~/.config/nordray-vault && chmod 700 ~/.config/nordray-vault
keepassxc-cli db-create --set-key-file ~/.config/nordray-vault/$APP.keyx "$TMP/$APP.kdbx"   # no password
```

1. The user adds the entries with `!`. Validate, then upload as `$APP.kdbx`.
2. On the Bunny app, the user sets the `NORDRAY_VAULT_*` variables, or you set them after their confirmation:
   - the key file as `base64 -i ~/.config/nordray-vault/$APP.keyx`;
   - the read-only password from `bunny storage zones credentials "$ZONE" --read-only`. It is the same for every app.

   Have the user pipe these into the Bunny dashboard or `bunny api` themselves, so they are never printed here.
3. The app's image must copy the binary and start with `nordray-vault -- …`.
4. Remind the user to save the key file in their password manager.

## Rotate a key file

```sh
keepassxc-cli db-edit --no-password -k "$KEY" --set-key-file "$KEY.new" "$TMP/$APP.kdbx"
```

1. Validate with the new key file, then upload.
2. Set `NORDRAY_VAULT_KEYFILE` on the app to the new key's base64 and restart the app.
3. `mv "$KEY.new" "$KEY"`, and update the password manager.

The backups made before the rotation still open with the old key only.

## Roll back

1. `bunny storage files list backups/ --zone "$ZONE"`, then download the copy you want.
2. Validate it.
3. After the user confirms, upload it as `$APP.kdbx` and restart the app.

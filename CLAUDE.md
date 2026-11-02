# CLAUDE.md

Read [README.md](README.md) first. Managing secrets: [skills/nordray-vault/SKILL.md](skills/nordray-vault/SKILL.md).

- This repository is public. Never commit a zone name, password, real key file, secret value or details about where
  it is deployed.
- Changes go through a pull request; `check` must pass. Dependabot keeps cargo, Actions and Docker updated.
- Never push `v*` tags: a `v*` tag publishes the image and a release. Only maintainers tag releases.
- No Rust toolchain is assumed locally: build and test with `docker build --target test .`.
- An error message must never contain a value, a key or a password, and a test checks this.

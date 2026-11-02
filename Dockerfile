# Alpine's Rust targets musl, so the release binary is static and runs in any Linux image.
FROM rust:1-alpine AS source
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

# CI runs the tests with `docker build --target test .`.
FROM source AS test
COPY tests/ tests/
RUN cargo test --locked

FROM source AS build
RUN cargo build --release --locked

# THIRD-PARTY-LICENSES: the license texts of every crate compiled into the binary, shipped next to it.
FROM rust:1-alpine AS licenses
RUN apk add --no-cache musl-dev && cargo install --locked --features cli cargo-about@0.9.3
WORKDIR /src
COPY Cargo.toml Cargo.lock about.toml about.hbs ./
COPY src/ src/
RUN cargo about generate --locked --fail about.hbs -o THIRD-PARTY-LICENSES

# Apps copy the binary: COPY --from=ghcr.io/activout/nordray-vault:<version> /nordray-vault /usr/local/bin/
FROM scratch
COPY --from=build /src/target/release/nordray-vault /nordray-vault
COPY --from=licenses /src/THIRD-PARTY-LICENSES /THIRD-PARTY-LICENSES
ENTRYPOINT ["/nordray-vault"]

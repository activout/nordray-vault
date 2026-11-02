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

# Apps copy the binary: COPY --from=ghcr.io/activout/nordray-vault:<version> /nordray-vault /usr/local/bin/
FROM scratch
COPY --from=build /src/target/release/nordray-vault /nordray-vault
ENTRYPOINT ["/nordray-vault"]

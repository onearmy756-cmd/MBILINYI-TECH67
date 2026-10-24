# syntax=docker/dockerfile:1
# ================================================================
#  Mbilinyi Tech Solutions — the Rust core, on Fly.io
#
#  The Rust service (`rust/mts-core`) answers every page and every
#  `/api/*` call, so it is the whole server. The PHP files are the
#  original shell and are not needed at runtime.
#
#  Stage 1 builds the binary. Stage 2 is a Debian runtime that also
#  carries the language toolchains the platform advertises — Python,
#  Rust, Go, Elixir, R, Node.js and C — because `langrun.rs` only
#  offers a language it can actually run.
# ================================================================

FROM rust:1-slim-bookworm AS build

# pqcrypto-* and rusqlite (bundled) compile C, so a C toolchain is required.
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src/rust/mts-core

# Dependencies first, so a change to a source file does not recompile them all.
COPY rust/mts-core/Cargo.toml rust/mts-core/Cargo.lock ./
RUN mkdir -p src && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY rust/mts-core/src ./src
RUN touch src/main.rs && cargo build --release

# ================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl gosu \
        python3 \
        gcc libc6-dev \
        nodejs npm \
        r-base-core \
        golang-go \
        elixir \
    && rm -rf /var/lib/apt/lists/*

# The Rust toolchain, so the "Rust" language in the platform really compiles.
COPY --from=build /usr/local/rustup /usr/local/rustup
COPY --from=build /usr/local/cargo /usr/local/cargo

# A non-root account: the platform runs code that visitors submit.
RUN useradd --create-home --shell /bin/bash mts \
    && mkdir -p /home/mts/go /home/mts/.cache/go-build /home/mts/.mix /data \
    && chown -R mts:mts /home/mts /data

WORKDIR /app

COPY --from=build /src/rust/mts-core/target/release/mts-core /app/mts-core
COPY templates /app/templates
COPY rust/mts-core/r /app/r
COPY assets /app/assets
COPY manifest.json sw.js robots.txt sitemap.xml /app/
# Outside /app on purpose: `static_file` serves any dotted path under the
# working directory, and this script has no business being downloadable.
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /app/mts-core

ENV MTS_PORT=8080 \
    MTS_DB=/data/mts.sqlite \
    MTS_TEMPLATES=/app/templates \
    MTS_R_DIR=/app/r \
    MTS_RSCRIPT=/usr/bin/Rscript \
    SITE_URL=https://mbilinyitech.co.tz \
    MTS_PUBLIC_URL=https://mbilinyitech.co.tz \
    MTS_RP_ID=mbilinyitech.co.tz \
    MTS_ORIGIN=https://mbilinyitech.co.tz \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    GOPATH=/home/mts/go \
    GOCACHE=/home/mts/.cache/go-build \
    MIX_HOME=/home/mts/.mix \
    PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

EXPOSE 8080
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["/app/mts-core"]

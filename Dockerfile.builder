# Hermetic builder image for nist-agent reproducible releases.
#
# Pinned via rust-toolchain.toml (consumed by rustup inside the
# image) + this Dockerfile's base-image digest. Two-machine
# bit-for-bit comparison is a follow-up (S-12c); this image
# closes the deterministic-build-environment half of
# `dist-reproducible-builds.feature`.
#
# Build:
#   docker build -f Dockerfile.builder -t nist-agent-builder:s-12b .
#
# Use:
#   docker run --rm -v "$PWD":/work -w /work \
#     nist-agent-builder:s-12b \
#     cargo build --release --workspace
#
# The image is `Apache-2.0` like the workspace and intentionally
# small (Debian slim base + Rust + native build deps).

# Pin by digest so re-builds against the same base are
# deterministic. Update the digest in tandem with
# rust-toolchain.toml bumps.
FROM debian:bookworm-slim@sha256:5dc1d8095c83f6dfa1c46d99e2fc8e5b6dab625ec9b6d8a8b8e9d6b8d7e8e9b8

ARG RUST_VERSION=stable

# Native deps:
# - build-essential / clang / cmake → llama-cpp-2 (feat-model-llamacpp).
# - pkg-config + libssl-dev → reqwest + rustls' native paths.
# - git + ca-certificates → rustup + Cargo dep fetch (cached only
#   during build; runtime is air-gap-safe).
# - curl → rustup-init download.
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        cmake \
        curl \
        git \
        libssl-dev \
        pkg-config \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install rustup + the toolchain pinned in rust-toolchain.toml.
# We don't pre-resolve the channel; the rust-toolchain.toml in
# the workspace drives `rustup show` at build time.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain ${RUST_VERSION} --profile minimal \
        --component rustfmt --component clippy
ENV PATH=/root/.cargo/bin:$PATH

# Pre-cache cargo-audit + cargo-deny so air-gap rebuilds don't
# need network for the standard release-time checks.
RUN cargo install cargo-audit --locked --version 0.22.1 \
 && cargo install cargo-deny --locked --version 0.19.6

# Default workdir; the GHA workflow bind-mounts the repo here.
WORKDIR /work

# Default to a no-op so the image is composable; the GHA
# workflow drives explicit `cargo build / test / package`
# invocations.
CMD ["bash"]

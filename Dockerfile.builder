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

# Pin by digest (multi-platform index) so re-builds against the
# same base are deterministic. Update the digest in tandem with
# rust-toolchain.toml bumps. Resolve a current digest with:
#   docker buildx imagetools inspect debian:bookworm-slim
FROM debian:bookworm-slim@sha256:0104b334637a5f19aa9c983a91b54c89887c0984081f2068983107a6f6c21eeb

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
        openssh-client \
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

# Pre-cache cargo-audit + cargo-deny + cargo-cyclonedx so the
# release pipeline performs no network tool install — the SBOM
# step runs entirely from this image (FUA-NIST-AGENT-03).
RUN cargo install cargo-audit --locked --version 0.22.1 \
 && cargo install cargo-deny --locked --version 0.19.6 \
 && cargo install cargo-cyclonedx --locked --version 0.5.7

# Default workdir; the GHA workflow bind-mounts the repo here.
WORKDIR /work

# Entrypoint chowns /host-ssh -> /root/.ssh so cargo's
# git-fetch-with-cli can resolve the federation host aliases.
# Silently skipped when /host-ssh isn't mounted (local ad-hoc).
COPY scripts/release/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]

# Default to a no-op so the image is composable; the GHA
# workflow drives explicit `cargo build / test / package`
# invocations.
CMD ["bash"]

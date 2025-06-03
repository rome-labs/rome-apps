# syntax=docker/dockerfile:1.4

FROM anzaxyz/agave:v2.1.7 as solana

FROM rust:1.86.0-slim as base

RUN apt-get update -qq && \
    apt-get install -y -qq --no-install-recommends \
    perl pkg-config libssl-dev \
    libpq-dev build-essential \
    curl protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

FROM base as builder
ARG FEATURE

ENV CARGO_HOME=/usr/local/cargo
ENV CARGO_TARGET_DIR=/opt/rome-apps/target

COPY ./rome-evm /opt/rome-evm
COPY ./rome-relayer /opt/rome-relayer
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-apps /opt/rome-apps

WORKDIR /opt/rome-apps/
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git,sharing=locked \
    --mount=type=cache,target=/opt/rome-apps/target,id=target-cache,sharing=locked \
    cargo fetch --locked --quiet

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git,sharing=locked \
    --mount=type=cache,target=/opt/rome-apps/target,id=target-cache,sharing=locked \
    RUSTFLAGS="-D warnings" cargo build --release --locked --quiet --features ${FEATURE} && \
    ls -l /opt/rome-apps/target/release/

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git,sharing=locked \
    --mount=type=cache,target=/opt/rome-apps/target,id=target-cache,sharing=locked \
    cargo install diesel_cli --no-default-features --features postgres && \
    mkdir -p /tmp/release/ && \
    cp /opt/rome-apps/target/release/proxy /tmp/release/ && \
    cp /opt/rome-apps/target/release/rhea /tmp/release/ && \
    cp /opt/rome-apps/target/release/cli /tmp/release/ && \
    cp /opt/rome-apps/target/release/hercules /tmp/release/ 


FROM base AS runtime

COPY --from=solana /usr/bin/solana /usr/bin/solana
COPY --from=builder /tmp/release/proxy /opt/
COPY --from=builder /tmp/release/rhea /opt/
COPY --from=builder /tmp/release/cli /opt/
COPY --from=builder /tmp/release/hercules /opt/
COPY --from=builder /usr/local/cargo/bin/diesel /opt/
COPY ./rome-apps/docker/entrypoint.sh /opt/
COPY ./rome-apps/docker/cli.sh /opt/
COPY ./rome-apps/docker/cli-deploy.sh /opt/
COPY ./rome-apps/docker/apply_migrations /opt/
COPY ./rome-sdk/rome-evm-client/src/indexer/pg_storage/migrations /opt/migrations

WORKDIR /opt

ENTRYPOINT ["./entrypoint.sh"]


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

COPY ./rome-evm /opt/rome-evm
COPY ./rome-relayer /opt/rome-relayer
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-apps /opt/rome-apps

WORKDIR /opt/rome-apps/

RUN RUSTFLAGS="-D warnings" cargo build --release --features ${FEATURE} --no-default-features && \
    ls -l /opt/rome-apps/target/release/
RUN cargo install diesel_cli --no-default-features --features postgres

FROM base AS runtime

COPY --from=solana /usr/bin/solana /usr/bin/solana
COPY --from=builder /usr/local/cargo/bin/diesel /opt
COPY --from=builder /opt/rome-apps/target/release/proxy /opt/
COPY --from=builder /opt/rome-apps/target/release/rhea /opt/
COPY --from=builder /opt/rome-apps/target/release/cli /opt/
COPY --from=builder /opt/rome-apps/target/release/hercules /opt/
COPY ./rome-apps/docker/entrypoint.sh /opt/
COPY ./rome-apps/docker/cli.sh /opt/
COPY ./rome-apps/docker/cli-deploy.sh /opt/
COPY ./rome-apps/docker/apply_migrations /opt/
COPY ./rome-sdk/rome-evm-client/src/indexer/pg_storage/migrations /opt/migrations

WORKDIR /opt

ENTRYPOINT ["./entrypoint.sh"]

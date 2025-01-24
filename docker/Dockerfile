FROM anzaxyz/agave:v2.1.7 as solana

FROM ubuntu:22.04 as base
RUN apt update && apt install -y cargo-1.79 pkg-config libssl-dev libpq-dev build-essential curl

FROM base as builder

COPY ./rome-evm /opt/rome-evm
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-apps /opt/rome-apps

WORKDIR /opt/rome-apps/
RUN RUSTFLAGS="-D warnings" cargo-1.79 build --release
RUN cargo-1.79 install diesel_cli --no-default-features --features postgres

FROM base AS runtime

COPY --from=solana /usr/bin/solana /usr/bin/solana
COPY --from=builder /opt/rome-apps/target/release/proxy /opt/
COPY --from=builder /opt/rome-apps/target/release/rhea /opt/
COPY --from=builder /opt/rome-apps/target/release/cli /opt/
COPY --from=builder /opt/rome-apps/target/release/hercules /opt/
COPY --from=builder /root/.cargo/bin/diesel /opt/
COPY ./rome-apps/docker/entrypoint.sh /opt/
COPY ./rome-apps/docker/cli.sh /opt/
COPY ./rome-apps/docker/cli-deploy.sh /opt/
COPY ./rome-apps/docker/apply_migrations /opt/
COPY ./rome-sdk/rome-evm-client/src/indexer/pg_storage/migrations /opt/migrations

WORKDIR /opt

ENTRYPOINT ["./entrypoint.sh"]


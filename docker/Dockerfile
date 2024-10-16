FROM solanalabs/solana:v1.18.17 AS solana

FROM ubuntu:22.04 as builder

RUN apt update && apt install -y rustc cargo pkg-config libssl-dev

COPY ./rome-evm /opt/rome-evm
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-apps /opt/rome-apps

WORKDIR /opt/rome-apps/
RUN RUSTFLAGS="-D warnings" cargo build --release

FROM ubuntu:22.04 AS runtime
RUN apt update && apt install -y ca-certificates wget libssl3 libssl-dev curl

COPY --from=solana /usr/bin/solana /usr/bin/solana
COPY --from=builder /opt/rome-apps/target/release/proxy /opt/
COPY --from=builder /opt/rome-apps/target/release/rhea /opt/
COPY --from=builder /opt/rome-apps/target/release/cli /opt/
COPY ./rome-apps/docker/entrypoint.sh /opt/
COPY ./rome-apps/docker/cli.sh /opt/

WORKDIR /opt

ENTRYPOINT ["./entrypoint.sh"]

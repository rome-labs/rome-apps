FROM solanalabs/solana:v1.18.17 AS solana

FROM ubuntu:22.04 as builder

RUN apt update && apt install -y rustc cargo pkg-config libssl-dev

COPY ./rome-evm /opt/rome-evm
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-apps /opt/rome-apps

WORKDIR /opt/rome-apps/
RUN CHAIN_ID=1001 CONTRACT_OWNER=8q76RPN5Tm6thVoQAUFhUP2diddGgtDLA6B6eShSazB2 cargo build --release

FROM ubuntu:22.04 AS runtime
RUN apt update && apt install -y ca-certificates wget libssl3 libssl-dev

COPY --from=solana /usr/bin/solana /usr/bin/solana
COPY --from=builder /opt/rome-apps/target/release/proxy /opt/
COPY --from=builder /opt/rome-apps/target/release/rhea /opt/
COPY --from=builder /opt/rome-apps/target/release/hercules /opt/
COPY ./rome-apps/docker/entrypoint.sh /opt/

WORKDIR /opt

ENTRYPOINT ["./entrypoint.sh"]

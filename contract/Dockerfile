FROM rust:1.57.0-buster as builder-contract

USER root

WORKDIR /build

RUN rustup target add wasm32-unknown-unknown
RUN apt update && apt install binaryen -y

COPY ./compile.sh ./
COPY ./proxy_reencryption ./proxy_reencryption

RUN ./compile.sh ./proxy_reencryption


FROM scratch AS export-stage
COPY --from=builder-contract /build/cw_proxy_reencryption.wasm ./cw_proxy_reencryption.wasm
FROM rust:1.53.0-buster as builder-contract

USER root

WORKDIR /build

RUN rustup target add wasm32-unknown-unknown
RUN apt update && apt install binaryen -y

COPY ./contract/ ./

RUN mkdir cosmwasm && cd cosmwasm && ../compile.sh ../proxy_reencryption

# ##################################

FROM python:3.9-buster as base

USER root

WORKDIR /workdir

RUN pip3 install --no-cache-dir --upgrade pip

COPY ./requirements.txt ./
RUN pip3 install --no-cache-dir -r ./requirements.txt

COPY ./ ./
RUN pip3 install --no-cache-dir -e .

COPY --from=builder-contract /build/cosmwasm/ ./cosmwasm

ENTRYPOINT [ "./scripts/run_tests.sh" ]
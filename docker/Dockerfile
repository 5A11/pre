FROM rust:1.57.0-buster as builder-contract

USER root

WORKDIR /build

RUN rustup target add wasm32-unknown-unknown
RUN apt update && apt install binaryen -y

COPY ./contract/ ./

RUN ./compile.sh ./proxy_reencryption

# ##################################

FROM python:3.9-buster as base

USER root

WORKDIR /workdir

RUN pip3 install --no-cache-dir --upgrade pip
RUN pip3 install --no-cache-dir --upgrade pipenv

COPY ./Pipfile ./
RUN pipenv install -d --skip-lock --system

COPY ./ ./
RUN pip3 install --no-cache-dir -e .

COPY --from=builder-contract /build/ ./contract

ENTRYPOINT [ "./scripts/run_tests.sh" ]
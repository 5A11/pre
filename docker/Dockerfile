FROM python:3.9-buster as base

USER root

WORKDIR /workdir

RUN pip3 install --no-cache-dir --upgrade pip

COPY ./requirements.txt ./

RUN pip3 install --no-cache-dir -r ./requirements.txt
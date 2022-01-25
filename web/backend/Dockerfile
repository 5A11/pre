FROM python:3
ENV PYTHONUNBUFFERED 1

# Only for installing PRE as sub dependency
# TODO: Remove it when PRE is published and can be included in requirements.txt
RUN mkdir /pre
WORKDIR /pre
COPY . /pre/
RUN pip install -e .

RUN mkdir /backend
WORKDIR /backend
COPY ./web/backend /backend/
RUN pip install -r requirements.txt
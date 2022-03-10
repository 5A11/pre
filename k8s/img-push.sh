#!/bin/sh


docker-compose build

TAG=$(git log -1 --format=%h)


docker tag k8s_fetchd:latest gcr.io/fetch-ai-images/colearn-test_fetchd:$TAG
docker push gcr.io/fetch-ai-images/colearn-test_fetchd:$TAG

docker tag pre-relayer:latest gcr.io/fetch-ai-images/colearn-pre-relayer:$TAG
docker push gcr.io/fetch-ai-images/colearn-pre-relayer:$TAG

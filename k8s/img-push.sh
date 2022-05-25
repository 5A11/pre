#!/bin/sh


docker-compose build

TAG=$(git log -1 --format=%h)

docker tag pre-relayer:latest gcr.io/fetch-ai-images/colearn-pre-relayer$SUFFIX:$TAG 
docker push gcr.io/fetch-ai-images/colearn-pre-relayer$SUFFIX:$TAG
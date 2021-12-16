#!/bin/bash
# requires docker >= 19.03
DOCKER_BUILDKIT=1 docker build  --output . .
